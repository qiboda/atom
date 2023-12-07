use std::collections::BTreeMap;

use bevy::utils::tracing::{self, span};
use serde_json::json;
use tracing_subscriber::Layer;

pub struct TerrainLayer {
    pretty: bool,
}

impl TerrainLayer {
    pub fn new() -> Self {
        Self { pretty: false }
    }

    pub fn with_pretty(self, pretty: bool) -> Self {
        Self { pretty }
    }
}

impl<S> Layer<S> for TerrainLayer
where
    S: tracing::Subscriber,
    S: for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        static mut ORDER_ID: u64 = 0;
        unsafe {
            ORDER_ID += 1;
        }

        // All of the span context
        let scope = ctx.event_scope(event).unwrap();
        let mut spans = vec![];
        for span in scope.from_root() {
            let extensions = span.extensions();
            let storage = extensions.get::<TerrainSpanFieldStorage>().unwrap();
            let field_data: &BTreeMap<String, serde_json::Value> = &storage.0;
            spans.push(serde_json::json!({
                "target": span.metadata().target(),
                "name": span.name(),
                "level": format!("{:?}", span.metadata().level()),
                "fields": field_data,
            }));
        }

        // All of the event context
        let mut fields = BTreeMap::new();
        let mut terrain_visitor = TerrainVisitor(&mut fields);
        event.record(&mut terrain_visitor);

        let thread_id = format!("{:?}", std::thread::current().id());
        let thread_id = thread_id.replace("ThreadId(", "").replace(")", "");

        let event_json;
        unsafe {
            event_json = json!({
                "order_id": ORDER_ID,
                "thread_id":  thread_id.parse::<u64>().unwrap(),
                "target": event.metadata().target(),
                "level": format!("{:?}",  event.metadata().level()),
                "name": event.metadata().name(),
                "fields": fields,
                "spans": spans,
            });
        }
        if self.pretty {
            println!("{}", serde_json::to_string_pretty(&event_json).unwrap());
        } else {
            println!("{}", serde_json::to_string(&event_json).unwrap());
        }
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Build our json object from the field values like we have been
        let mut fields = BTreeMap::new();
        let mut visitor = TerrainVisitor(&mut fields);
        attrs.record(&mut visitor);

        // And stuff it in our newtype.
        let storage = TerrainSpanFieldStorage(fields);

        // Get a reference to the internal span data
        let span = ctx.span(id).unwrap();
        // Get the special place where tracing stores custom data
        let mut extensions = span.extensions_mut();
        // And store our data
        extensions.insert::<TerrainSpanFieldStorage>(storage);
    }

    fn on_record(
        &self,
        span: &span::Id,
        values: &span::Record<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Get the span whose data is being recorded
        let span = ctx.span(span).unwrap();

        // Get a mutable reference to the data we created in new_span
        let mut extensions_mut = span.extensions_mut();
        let custom_field_storage: &mut TerrainSpanFieldStorage =
            extensions_mut.get_mut::<TerrainSpanFieldStorage>().unwrap();
        let json_data: &mut BTreeMap<String, serde_json::Value> = &mut custom_field_storage.0;

        // And add to using our old friend the visitor!
        let mut visitor = TerrainVisitor(json_data);
        values.record(&mut visitor);
    }
}

struct TerrainVisitor<'a>(&'a mut BTreeMap<String, serde_json::Value>);

impl<'a> tracing::field::Visit for TerrainVisitor<'a> {
    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(format!("{:?}", value)),
        );
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(format!("{:?}", value)),
        );
    }
}

#[derive(Debug)]
struct TerrainSpanFieldStorage(BTreeMap<String, serde_json::Value>);

#[cfg(test)]
mod test {
    use bevy::{
        log::{debug_span, info_span},
        utils::tracing,
    };
    use tracing_subscriber::layer::SubscriberExt;

    use crate::terrain::trace::terrain_tracing::TerrainLayer;

    #[test]
    fn test_custom_layer() {
        let layer = TerrainLayer::new();
        let subscriber = tracing_subscriber::registry().with(layer.with_pretty(true));
        tracing::subscriber::set_global_default(subscriber).unwrap();

        let outer_span = info_span!("outer", level = 0);
        let _outer_entered = outer_span.enter();

        let inner_span = debug_span!("inner", level = 1);
        let _inner_entered = inner_span.enter();

        let test = "saldfjlas";
        let test2 = "saldfjlas";
        let test3 = "saldfjlas";
        tracing::info!(test, test2, test3);
        tracing::info!(test, test2, test3);
    }
}
