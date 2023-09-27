use std::ops::{Deref, Not};

use bevy::{prelude::Component, utils::HashMap};
use layertag::layertag::LayerTag;

#[derive(Debug, Component, Default)]
pub struct AbilityTagContainer {
    pub ability_tag_map: HashMap<String, AbilityTag>,
}

#[derive(Debug, Default)]
pub enum AbilityTagFilter {
    #[default]
    Enable,
    Disable,
    EnableAndDisable,
}

#[derive(Debug, Default)]
pub struct AbilityTag {
    enable: Vec<Box<dyn LayerTag>>,
    disable: Vec<Box<dyn LayerTag>>,
    ability_tag_filter: AbilityTagFilter,
}

impl AbilityTag {
    pub fn set_ability_tag_filter(&mut self, ability_tag_filter: AbilityTagFilter) {
        self.ability_tag_filter = ability_tag_filter;
    }

    pub fn add_enable_tag(&mut self, tag: Box<dyn LayerTag>) {
        assert!(self
            .disable
            .iter()
            .any(|x| tag.deref().exact_match((*x).deref()))
            .not());
        self.enable.push(tag);
    }

    pub fn remove_enable_tag(&mut self, tag: Box<dyn LayerTag>) {
        self.enable
            .retain(|x| x.deref().exact_match(tag.deref()).not());
    }

    pub fn add_disable_tag(&mut self, tag: Box<dyn LayerTag>) {
        assert!(self
            .enable
            .iter()
            .any(|x| tag.deref().exact_match((*x).deref()))
            .not());
        self.disable.push(tag);
    }

    pub fn remove_disable_tag(&mut self, tag: Box<dyn LayerTag>) {
        self.disable
            .retain(|x| x.deref().exact_match(tag.deref()).not());
    }
}

impl AbilityTag {
    pub fn is_enable_custom<F>(&self, tag: &dyn LayerTag, condition: &F) -> bool
    where
        F: Fn(&dyn LayerTag, &dyn LayerTag, AbilityTagFilter) -> bool,
    {
        self.enable.iter().any(|x| {
            tag.exact_match((*x).deref())
                && tag.cmp_data_same_type(x.deref())
                && condition(tag, x.deref(), AbilityTagFilter::Enable)
        })
    }

    pub fn is_disable_custom<F>(&self, tag: &dyn LayerTag, condition: &F) -> bool
    where
        F: Fn(&dyn LayerTag, &dyn LayerTag, AbilityTagFilter) -> bool,
    {
        self.disable.iter().any(|x| {
            tag.exact_match((*x).deref())
                && tag.cmp_data_same_type(x.deref())
                && condition(tag, x.deref(), AbilityTagFilter::Disable)
        })
    }

    pub fn check_pass_custom<F>(&self, tag: &dyn LayerTag, condition: &F) -> bool
    where
        F: Fn(&dyn LayerTag, &dyn LayerTag, AbilityTagFilter) -> bool,
    {
        match self.ability_tag_filter {
            AbilityTagFilter::Enable => self.is_enable_custom(tag, condition),
            AbilityTagFilter::Disable => self.is_disable_custom(tag, condition),
            AbilityTagFilter::EnableAndDisable => {
                self.is_enable_custom(tag, condition)
                    && self.is_disable_custom(tag, condition).not()
            }
        }
    }

    pub fn is_enable(&self, tag: &dyn LayerTag) -> bool {
        self.is_enable_custom(tag, &|_, _, _| true)
    }

    pub fn is_disable(&self, tag: &dyn LayerTag) -> bool {
        self.is_disable_custom(tag, &|_, _, _| true)
    }

    pub fn check_pass(&self, tag: &dyn LayerTag) -> bool {
        self.check_pass_custom(tag, &|_, _, _| true)
    }
}
