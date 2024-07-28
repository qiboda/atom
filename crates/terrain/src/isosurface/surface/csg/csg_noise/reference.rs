/// Planet seed. Change this to generate a different planet.
const CURRENT_SEED: u32 = 0;

/// Frequency of the planet's continents. Higher frequency produces
/// smaller, more numerous continents. This value is measured in radians.
const CONTINENT_FREQUENCY: f64 = 1.0;

/// Lacunarity of the planet's continents. Changing this value produces
/// slightly different continents. For the best results, this value should
/// be random, but close to 2.0.
const CONTINENT_LACUNARITY: f64 = 2.208984375;

/// Lacunarity of the planet's mountains. Changing the value produces
/// slightly different mountains. For the best results, this value should
/// be random, but close to 2.0.
const MOUNTAIN_LACUNARITY: f64 = 2.142578125;

/// Lacunarity of the planet's hills. Changing this value produces
/// slightly different hills. For the best results, this value should be
/// random, but close to 2.0.
const HILLS_LACUNARITY: f64 = 2.162109375;

/// Lacunarity of the planet's plains. Changing this value produces
/// slightly different plains. For the best results, this value should be
/// random, but close to 2.0.
const PLAINS_LACUNARITY: f64 = 2.314453125;

/// Lacunarity of the planet's badlands. Changing this value produces
/// slightly different badlands. For the best results, this value should
/// be random, but close to 2.0.
const BADLANDS_LACUNARITY: f64 = 2.212890625;

/// Specifies the "twistiness" of the mountains.
const MOUNTAINS_TWIST: f64 = 1.0;

/// Specifies the "twistiness" of the hills.
const HILLS_TWIST: f64 = 1.0;

/// Specifies the "twistiness" of the badlands.
const BADLANDS_TWIST: f64 = 1.0;

/// Specifies the planet's sea level. This value must be between -1.0
/// (minimum planet elevation) and +1.0 (maximum planet elevation).
const SEA_LEVEL: f64 = 0.0;

/// Specifies the level on the planet in which continental shelves appear.
/// This value must be between -1.0 (minimum planet elevation) and +1.0
/// (maximum planet elevation), and must be less than `SEA_LEVEL`.
const SHELF_LEVEL: f64 = -0.375;

/// Determines the amount of mountainous terrain that appears on the
/// planet. Values range from 0.0 (no mountains) to 1.0 (all terrain is
/// covered in mountains). Mountains terrain will overlap hilly terrain.
/// Because the badlands terrain may overlap parts of the mountainous
/// terrain, setting `MOUNTAINS_AMOUNT` to 1.0 may not completely cover the
/// terrain in mountains.
const MOUNTAINS_AMOUNT: f64 = 0.5;

/// Determines the amount of hilly terrain that appears on the planet.
/// Values range from 0.0 (no hills) to 1.0 (all terrain is covered in
/// hills). This value must be less than `MOUNTAINS_AMOUNT`. Because the
/// mountains terrain will overlap parts of the hilly terrain, and the
/// badlands terrain may overlap parts of the hilly terrain, setting
/// `HILLS_AMOUNT` to 1.0 may not completely cover the terrain in hills.
const HILLS_AMOUNT: f64 = (1.0 + MOUNTAINS_AMOUNT) / 2.0;

/// Determines the amount of badlands terrain that covers the planet.
/// Values range from 0.0 (no badlands) to 1.0 (all terrain is covered in
/// badlands). Badlands terrain will overlap any other type of terrain.
const BADLANDS_AMOUNT: f64 = 0.3125;

/// Offset to apply to the terrain type definition. Low values (< 1.0)
/// cause the rough areas to appear only at high elevations. High values
/// (> 2.0) cause the rough areas to appear at any elevation. The
/// percentage of rough areas on the planet are independent of this value.
const TERRAIN_OFFSET: f64 = 1.0;

/// Specifies the amount of "glaciation" on the mountains. This value
/// should be close to 1.0 and greater than 1.0.
const MOUNTAIN_GLACIATION: f64 = 1.375;

/// Scaling to apply to the base continent elevations, in planetary
/// elevation units.
const CONTINENT_HEIGHT_SCALE: f64 = (1.0 - SEA_LEVEL) / 4.0;

/// Maximum depth of the rivers, in planetary elevation units.
const RIVER_DEPTH: f64 = 0.0234375;

use std::path::Path;

use noise::core::worley::ReturnType;
use noise::MultiFractal;
use noise::Seedable;

use crate::isosurface::surface::csg::aworley::AWorley;

use noise::Billow;

use noise::Exponent;

use noise::Add;

use noise::Multiply;

use noise::Max;

use noise::Blend;

use noise::Constant;

use noise::RidgedMulti;

use noise::Terrace;

use noise::Select;

use noise::Turbulence;

use crate::isosurface::surface::csg::arc_noise::ArcNoise;

use noise::utils::PlaneMapBuilder;

use crate::isosurface::surface::csg::noise_cache::ACache;

use noise::Clamp;

use noise::Min;

use noise::ScaleBias;

use noise::Curve;

use noise::Perlin;

use noise::Fbm;

use noise::NoiseFn;

// ////////////////////////////////////////////////////////////////////////
// Function subgroup: base continent definition (7 noise functions)
//
// This subgroup roughly defines the positions and base elevations of the
// planet's continents.
//
// The "base elevation" is the elevation of the terrain before any terrain
// features (mountains, hills, etc.) are placed on that terrain.
//
// -1.0 represents the lowest elevations and +1.0 represents the highest
// elevations.
//
pub(crate) fn base_continent_def() -> impl NoiseFn<f64, 2> + Clone {
    // 1: [Continent module]: This FBM module generates the continents. This
    // noise function has a high number of octaves so that detail is visible at
    // high zoom levels.
    let base_continent_def_fb0 = Fbm::<Perlin>::new(CURRENT_SEED)
        .set_frequency(CONTINENT_FREQUENCY)
        .set_persistence(0.5)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(14);

    //    debug::render_noise_module("complexplanet_images/00_0_baseContinentDef_fb0\
    //    .png",
    //                               &baseContinentDef_fb0,
    //                               1024,
    //                               1024,
    //                               100);

    // 2: [Continent-with-ranges module]: Next, a curve module modifies the
    // output value from the continent module so that very high values appear
    // near sea level. This defines the positions of the mountain ranges.
    let base_continent_def_cu = Curve::new(base_continent_def_fb0)
        .add_control_point(-2.0000 + SEA_LEVEL, -1.625 + SEA_LEVEL)
        .add_control_point(-1.0000 + SEA_LEVEL, -1.375 + SEA_LEVEL)
        .add_control_point(0.0000 + SEA_LEVEL, -0.375 + SEA_LEVEL)
        .add_control_point(0.0625 + SEA_LEVEL, 0.125 + SEA_LEVEL)
        .add_control_point(0.1250 + SEA_LEVEL, 0.250 + SEA_LEVEL)
        .add_control_point(0.2500 + SEA_LEVEL, 1.000 + SEA_LEVEL)
        .add_control_point(0.5000 + SEA_LEVEL, 0.250 + SEA_LEVEL)
        .add_control_point(0.7500 + SEA_LEVEL, 0.250 + SEA_LEVEL)
        .add_control_point(1.0000 + SEA_LEVEL, 0.500 + SEA_LEVEL)
        .add_control_point(2.0000 + SEA_LEVEL, 0.500 + SEA_LEVEL);

    //    debug::render_noise_module("complexplanet_images/00_1_baseContinentDef_cu\
    //    .png",
    //                               &baseContinentDef_cu,
    //                               1024,
    //                               1024,
    //                               100);

    // 3: [Carver module]: This higher-frequency BasicMulti module will be
    // used by subsequent noise functions to carve out chunks from the
    // mountain ranges within the continent-with-ranges module so that the
    // mountain ranges will not be completely impassible.
    let base_continent_def_fb1 = Fbm::<Perlin>::new(CURRENT_SEED + 1)
        .set_frequency(CONTINENT_FREQUENCY * 4.34375)
        .set_persistence(0.5)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(11);

    //    debug::render_noise_module("complexplanet_images/00_2_baseContinentDef_fb1\
    //    .png",
    //                               &baseContinentDef_fb1,
    //                               1024,
    //                               1024,
    //                               100);

    // 4: [Scaled-carver module]: This scale/bias module scales the output
    // value from the carver module such that it is usually near 1.0. This
    // is required for step 5.
    let base_continent_def_sb = ScaleBias::new(base_continent_def_fb1)
        .set_scale(0.375)
        .set_bias(0.625);

    //    debug::render_noise_module("complexplanet_images/00_3_baseContinentDef_sb\
    //    .png",
    //                               &baseContinentDef_sb,
    //                               1024,
    //                               1024,
    //                               100);

    // 5: [Carved-continent module]: This minimum-value module carves out
    // chunks from the continent-with-ranges module. it does this by ensuring
    // that only the minimum of the output values from the scaled-carver
    // module and the continent-with-ranges module contributes to the output
    // value of this subgroup. Most of the time, the minimum value module will
    // select the output value from the continent-with-ranges module since the
    // output value from the scaled-carver is usually near 1.0. Occasionally,
    // the output from the scaled-carver module will be less than the output
    // value from the continent-with-ranges module, so in this case, the output
    // value from the scaled-carver module is selected.
    let base_continent_def_mi = Min::new(base_continent_def_sb, base_continent_def_cu);

    //    debug::render_noise_module("complexplanet_images/00_4_baseContinentDef_mi\
    //    .png",
    //                               &baseContinentDef_mi,
    //                               1024,
    //                               1024,
    //                               100);

    // 6: [Clamped-continent module]: Finally, a clamp module modifies the
    // carved continent module to ensure that the output value of this subgroup
    // is between -1.0 and 1.0.
    let base_continent_def_cl: Clamp<_, _, 2> =
        Clamp::new(base_continent_def_mi).set_bounds(-1.0, 1.0);

    // 7: [Base-continent-definition subgroup]: ACaches the output value from
    // the clamped-continent module.
    let base_continent_def = ACache::new(base_continent_def_cl);

    let noise_map = PlaneMapBuilder::new_fn(|x| base_continent_def.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("base_continent_def.png"));

    base_continent_def
}

pub fn continent_def(base_continent_def: ArcNoise<f64, 2>) -> impl NoiseFn<f64, 2> + Clone {
    //    debug::render_noise_module("complexplanet_images/00_5_baseContinentDef.png",
    //                               &baseContinentDef,
    //                               1024,
    //                               1024,
    //                               100);

    // ////////////////////////////////////////////////////////////////////////
    // Function subgroup: continent definition (5 noise functions)
    //
    // This subgroup warps the output value from the base-continent-definition
    // subgroup, producing more realistic terrain.
    //
    // Warping the base continent definition produces lumpier terrain with
    // cliffs and rifts.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Coarse-turbulence module]: This turbulence module warps the output
    // value from the base-continent-definition subgroup, adding some coarse
    // detail to it.
    let continent_def_tu0 = Turbulence::<_, Perlin>::new(base_continent_def.clone())
        .set_seed(CURRENT_SEED + 10)
        .set_frequency(CONTINENT_FREQUENCY * 15.25)
        .set_power(CONTINENT_FREQUENCY / 113.75)
        .set_roughness(13);

    //    debug::render_noise_module("complexplanet_images/01_0_continentDef_tu0.png",
    //                               &continentDef_tu0,
    //                               1024,
    //                               1024,
    //                               1000);

    // 2: [Intermediate-turbulence module]: This turbulence module warps the
    // output value from the coarse-turbulence module. This turbulence has a
    // higher frequency, but lower power, than the coarse-turbulence module,
    // adding some intermediate detail to it.
    let continent_def_tu1 = Turbulence::<_, Perlin>::new(continent_def_tu0)
        .set_seed(CURRENT_SEED + 11)
        .set_frequency(CONTINENT_FREQUENCY * 47.25)
        .set_power(CONTINENT_FREQUENCY / 433.75)
        .set_roughness(12);

    //    debug::render_noise_module("complexplanet_images/01_1_continentDef_tu1.png",
    //                               &continentDef_tu1,
    //                               1024,
    //                               1024,
    //                               1000);

    // 3: [Warped-base-continent-definition module]: This turbulence module
    // warps the output value from the intermediate-turbulence module. This
    // turbulence has a higher frequency, but lower power, than the
    // intermediate-turbulence module, adding some fine detail to it.
    let continent_def_tu2 = Turbulence::<_, Perlin>::new(continent_def_tu1)
        .set_seed(CURRENT_SEED + 12)
        .set_frequency(CONTINENT_FREQUENCY * 95.25)
        .set_power(CONTINENT_FREQUENCY / 1019.75)
        .set_roughness(11);

    //    debug::render_noise_module("complexplanet_images/01_2_continentDef_tu2.png",
    //                               &continentDef_tu2,
    //                               1024,
    //                               1024,
    //                               1000);

    // 4: [Select-turbulence module]: At this stage, the turbulence is applied
    // to the entire base-continent-definition subgroup, producing some very
    // rugged, unrealistic coastlines.  This selector module selects the
    // output values from the (unwarped) base-continent-definition subgroup
    // and the warped-base-continent-definition module, based on the output
    // value from the (unwarped) base-continent-definition subgroup.  The
    // selection boundary is near sea level and has a relatively smooth
    // transition.  In effect, only the higher areas of the base-continent-
    // definition subgroup become warped; the underwater and coastal areas
    // remain unaffected.
    let continent_def_se = Select::new(
        base_continent_def.clone(),
        continent_def_tu2,
        base_continent_def.clone(),
    )
    .set_bounds(SEA_LEVEL - 0.0375, SEA_LEVEL + 1000.0375)
    .set_falloff(0.0625);

    //    debug::render_noise_module("complexplanet_images/01_3_continentDef_se.png",
    //                               &continentDef_se,
    //                               1024,
    //                               1024,
    //                               1000);

    // 5: [Continent-definition group]: ACaches the output value from the
    // clamped-continent module. This is the output value for the entire
    // continent-definition group.
    let continent_def = ACache::new(continent_def_se.clone());

    //    debug::render_noise_module("complexplanet_images/01_4_continentDef.png",
    //                               &continentDef,
    //                               1024,
    //                               1024,
    //                               1000);
    let noise_map = PlaneMapBuilder::new_fn(|x| continent_def.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("continent_def.png"));

    continent_def
}

pub fn terrain_type_def(
    continent_def: impl NoiseFn<f64, 2> + Clone,
) -> impl NoiseFn<f64, 2> + Clone {
    // ////////////////////////////////////////////////////////////////////////
    // Function group: terrain type definition
    // ////////////////////////////////////////////////////////////////////////

    // ////////////////////////////////////////////////////////////////////////
    // Function subgroup: terrain type definition (3 noise functions)
    //
    // This subgroup defines the positions of the terrain types on the planet.
    //
    // Terrain types include, in order of increasing roughness, plains, hills,
    // and mountains.
    //
    // This subgroup's output value is based on the output value from the
    // continent-definition group. Rougher terrain mainly appears at higher
    // elevations.
    //
    // -1.0 represents the smoothest terrain types (plains and underwater) and
    // +1.0 represents the roughest terrain types (mountains).
    //

    // 1: [Warped-continent module]: This turbulence module slightly warps the
    // output value from the continent-definition group. This prevents the
    // rougher terrain from appearing exclusively at higher elevations. Rough
    // areas may now appear in the the ocean, creating rocky islands and
    // fjords.
    let terrain_type_def_tu = Turbulence::<_, Perlin>::new(continent_def.clone())
        .set_seed(CURRENT_SEED + 20)
        .set_frequency(CONTINENT_FREQUENCY * 18.125)
        .set_power(CONTINENT_FREQUENCY / 20.59375 * TERRAIN_OFFSET)
        .set_roughness(3);

    // 2: [Roughness-probability-shift module]: This terracing module sharpens
    // the edges of the warped-continent module near sea level and lowers the
    // slope towards the higher-elevation areas. This shrinks the areas in
    // which the rough terrain appears, increasing the "rarity" of rough
    // terrain.
    let terrain_type_def_te = Terrace::new(terrain_type_def_tu)
        .add_control_point(-1.00)
        .add_control_point(SHELF_LEVEL + SEA_LEVEL / 2.0)
        .add_control_point(1.00);

    // 3: [Terrain-type-definition group]: ACaches the output value from the
    // roughness-probability-shift module. This is the output value for the
    // entire terrain-type-definition group.

    let noise_map = PlaneMapBuilder::new_fn(|x| terrain_type_def_te.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("terrain_type_def.png"));

    ACache::new(terrain_type_def_te)
}

pub fn mountainous_base_def() -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function group: mountainous terrain
    // /////////////////////////////////////////////////////////////////////////

    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: mountain base definition (9 noise functions)
    //
    // This subgroup generates the base-mountain elevations. Other subgroups
    // will add the ridges and low areas to the base elevations.
    //
    // -1.0 represents low mountainous terrain and +1.0 represents high
    // mountainous terrain.
    //

    // 1: [Mountain-ridge module]: This ridged-multifractal-noise function
    // generates the mountain ridges.
    let mountain_base_def_rm0 = RidgedMulti::<Perlin>::new(CURRENT_SEED + 30)
        .set_frequency(1723.0)
        .set_lacunarity(MOUNTAIN_LACUNARITY)
        .set_octaves(4);

    // 2: [Scaled-mountain-ridge module]: Next, a scale/bias module scales the
    // output value from the mountain-ridge module so that its ridges are not
    // too high. The reason for this is that another subgroup adds actual
    // mountainous terrain to these ridges.
    let mountain_base_def_sb0: ScaleBias<_, _, 2> = ScaleBias::new(mountain_base_def_rm0)
        .set_scale(0.5)
        .set_bias(0.375);

    // 3: [River-valley module]: This ridged-multifractal-noise function
    // generates the river valleys.  It has a much lower frequency than the
    // mountain-ridge module so that more mountain ridges will appear outside
    // of the valleys. Note that this noise function generates ridged-multifractal
    // noise using only one octave; this information will be important in the
    // next step.
    let mountain_base_def_rm1 = RidgedMulti::<Perlin>::new(CURRENT_SEED + 31)
        .set_frequency(367.0)
        .set_lacunarity(MOUNTAIN_LACUNARITY)
        .set_octaves(1);

    // 4: [Scaled-river-valley module]: Next, a scale/bias module applies a
    // scaling factor of -2.0 to the output value from the river-valley module.
    // This stretches the possible elevation values because one-octave ridged-
    // multifractal noise has a lower range of output values than multiple-
    // octave ridged-multifractal noise. The negative scaling factor inverts
    // the range of the output value, turning the ridges from the river-valley
    // module into valleys.
    let mountain_base_def_sb1 = ScaleBias::new(mountain_base_def_rm1)
        .set_scale(-2.0)
        .set_bias(-0.5);

    // 5: [Low-flat module]: This low constant value is used by step 6.
    let mountain_base_def_co = Constant::new(-1.0);

    // 6: [Mountains-and-valleys module]: This blender module merges the scaled-
    // mountain-ridge module and the scaled-river-valley module together. It
    // causes the low-lying areas of the terrain to become smooth, and causes
    // the high-lying areas of the terrain to contain ridges. To do this, it
    // uses the scaled-river-valley module as the control module, causing the
    // low-flat module to appear in the lower areas and causing the scaled-
    // mountain-ridge module to appear in the higher areas.
    let mountain_base_def_bl = Blend::new(
        mountain_base_def_co,
        mountain_base_def_sb0,
        mountain_base_def_sb1,
    );

    // 7: [Coarse-turbulence module]: This turbulence module warps the output
    // value from the mountain-and-valleys module, adding some coarse detail to
    // it.
    let mountain_base_def_tu0 = Turbulence::<_, Perlin>::new(mountain_base_def_bl)
        .set_seed(CURRENT_SEED + 32)
        .set_frequency(1337.0)
        .set_power(1.0 / 6730.0 * MOUNTAINS_TWIST)
        .set_roughness(4);

    // 8: [Warped-mountains-and-valleys module]: This turbulence module warps
    // the output value from the coarse-turbulence module. This turbulence has
    // a higher frequency, but lower power, than the coarse-turbulence module,
    // adding some fine detail to it.
    let mountain_base_def_tu1 = Turbulence::<_, Perlin>::new(mountain_base_def_tu0)
        .set_seed(CURRENT_SEED + 33)
        .set_frequency(21221.0)
        .set_power(1.0 / 120157.0 * MOUNTAINS_TWIST)
        .set_roughness(6);

    // 9: [Mountain-base-definition subgroup]: ACaches the output value from the
    // warped-mountains-and-valleys module.

    let noise_map = PlaneMapBuilder::new_fn(|x| mountain_base_def_tu1.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("mountainous_base_def.png"));

    ACache::new(mountain_base_def_tu1)
}

pub fn high_mountainous_terrain() -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: high mountainous terrain (5 noise functions)
    //
    // This subgroup generates the mountainous terrain that appears at high
    // elevations within the mountain ridges.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Mountain-basis-0 module]: This ridged-multifractal-noise function,
    // along with the mountain-basis-1 module, generates the individual
    // mountains.
    let mountainous_high_rm0 = RidgedMulti::<Perlin>::new(CURRENT_SEED + 40)
        .set_frequency(2371.0)
        .set_lacunarity(MOUNTAIN_LACUNARITY)
        .set_octaves(3);

    // 2: [Mountain-basis-1 module]: This ridged-multifractal-noise function,
    // along with the mountain-basis-0 module, generates the individual
    // mountains.
    let mountainous_high_rm1 = RidgedMulti::<Perlin>::new(CURRENT_SEED + 41)
        .set_frequency(2341.0)
        .set_lacunarity(MOUNTAIN_LACUNARITY)
        .set_octaves(3);

    // 3: [High-mountains module]: Next, a maximum-value module causes more
    // mountains to appear at the expense of valleys. It does this by ensuring
    // that only the maximum of the output values from the two ridged-
    // multifractal-noise functions contribute to the output value of this
    // subgroup.
    let mountainous_high_ma: Max<_, _, _, 2> = Max::new(mountainous_high_rm0, mountainous_high_rm1);

    // 4: [Warped-high-mountains module]: This turbulence module warps the
    // output value from the high-mountains module, adding some detail to it.
    let mountainous_high_tu = Turbulence::<_, Perlin>::new(mountainous_high_ma)
        .set_seed(CURRENT_SEED + 42)
        .set_frequency(31511.0)
        .set_power(1.0 / 180371.0 * MOUNTAINS_TWIST)
        .set_roughness(4);

    // 5: [High-mountainous-terrain subgroup]: ACaches the output value from the
    // warped-high-mountains module.

    let noise_map = PlaneMapBuilder::new_fn(|x| mountainous_high_tu.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("high_mountainous_terrain.png"));

    ACache::new(mountainous_high_tu)
}

pub fn low_mountainous_terrain() -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: low mountainous terrain (4 noise functions)
    //
    // This subgroup generates the mountainous terrain that appears at low
    // elevations within the river valleys.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Lowland-basis-0 module]: This ridged-multifractal-noise function,
    // along with the lowland-basis-1 module, produces the low mountainous
    // terrain.
    let mountainous_low_rm0 = RidgedMulti::<Perlin>::new(CURRENT_SEED + 50)
        .set_frequency(1381.0)
        .set_lacunarity(MOUNTAIN_LACUNARITY)
        .set_octaves(8);

    // 1: [Lowland-basis-1 module]: This ridged-multifractal-noise function,
    // along with the lowland-basis-0 module, produces the low mountainous
    // terrain.
    let mountainous_low_rm1 = RidgedMulti::<Perlin>::new(CURRENT_SEED + 51)
        .set_frequency(1427.0)
        .set_lacunarity(MOUNTAIN_LACUNARITY)
        .set_octaves(8);

    // 3: [Low-mountainous-terrain module]: This multiplication module combines
    // the output values from the two ridged-multifractal-noise functions. This
    // causes the following to appear in the resulting terrain:
    // - Cracks appear when two negative output values are multiplied together.
    // - Flat areas appear when a positive and a negative output value are
    //   multiplied together.
    // - Ridges appear when two positive output values are multiplied together.
    let mountainous_low_mu: Multiply<_, _, _, 2> =
        Multiply::new(mountainous_low_rm0, mountainous_low_rm1);

    // 4: [Low-mountainous-terrain subgroup]: ACaches the output value from the
    // low-mountainous-terrain module.

    let noise_map = PlaneMapBuilder::new_fn(|x| mountainous_low_mu.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("low_mountainous_terrain.png"));

    ACache::new(mountainous_low_mu)
}

pub fn mountainous_terrain(
    mountainous_base_def: ArcNoise<f64, 2>,
    high_mountainous: ArcNoise<f64, 2>,
    low_mountainous: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: mountainous terrain (7 noise functions)
    //
    // This subgroup generates the final mountainous terrain by combining the
    // high-mountainous-terrain subgroup with the low-mountainous-terrain
    // subgroup.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Scaled-low-mountainous-terrain module]: First, this scale/bias module
    // scales the output value from the low-mountainous-terrain subgroup to a very
    // low value and biases it towards -1.0. This results in the low mountainous
    // areas becoming more-or-less flat with little variation. This will also
    // result in the low mountainous areas appearing at the lowest elevations in
    // this subgroup.
    let mountainous_terrain_sb0 = ScaleBias::new(low_mountainous)
        .set_scale(0.03125)
        .set_bias(-0.96875);

    // 2: [Scaled-high-mountainous-terrain module]: Next, this scale/bias module
    // scales the output value from the high-mountainous-terrain subgroup to 1/4
    // of its initial value and biases it so that its output value is usually
    // positive.
    let mountainous_terrain_sb1 = ScaleBias::new(high_mountainous)
        .set_scale(0.25)
        .set_bias(0.25);

    // 3: [Added-high-mountainous-terrain module]: This addition module adds the
    // output value from the scaled-high-mountainous-terrain module to the
    // output value from the mountain-base-definition subgroup. Mountains now
    // appear all over the terrain.
    let mountainous_terrain_ad = Add::new(mountainous_terrain_sb1, mountainous_base_def.clone());

    // 4: [Combined-mountainous-terrain module]: Note that at this point, the
    // entire terrain is covered in high mountainous terrain, even at the low
    // elevations. To make sure the mountains only appear at the higher
    // elevations, this selector module causes low mountainous terrain to appear
    // at the low elevations (within the valleys) and the high mountainous
    // terrain to appear at the high elevations (within the ridges). To do this,
    // this noise function selects the output value from the added-high-
    // mountainous-terrain module if the output value from the mountain-base-
    // definition subgroup is higher than a set amount. Otherwise, this noise
    // module selects the output value from the scaled-low-mountainous-terrain
    // module.
    let mountainous_terrain_se = Select::new(
        mountainous_terrain_sb0,
        mountainous_terrain_ad,
        mountainous_base_def,
    )
    .set_bounds(-0.5, 999.5)
    .set_falloff(0.5);

    // 5: [Scaled-mountainous-terrain-module]: This scale/bias module slightly
    // reduces the range of the output value from the combined-mountainous-
    // terrain module, decreasing the heights of the mountain peaks.
    let mountainous_terrain_sb2 = ScaleBias::new(mountainous_terrain_se)
        .set_scale(0.8)
        .set_bias(0.0);

    // 6: [Glaciated-mountainous-terrain-module]: This exponential-curve module
    // applies an exponential curve to the output value from the scaled-
    // mountainous-terrain module. This causes the slope of the mountains to
    // smoothly increase towards higher elevations, as if a glacier ground out
    // those mountains. This exponential-curve module expects the output value
    // to range from -1.0 to +1.0.
    let mountainous_terrain_ex =
        Exponent::new(mountainous_terrain_sb2).set_exponent(MOUNTAIN_GLACIATION);

    let noise_map = PlaneMapBuilder::new_fn(|x| mountainous_terrain_ex.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("mountainous_terrain.png"));

    ACache::new(mountainous_terrain_ex)
}

pub fn hilly_terrain() -> impl NoiseFn<f64, 2> + Clone {
    // ////////////////////////////////////////////////////////////////////////
    // Function group: hilly terrain
    // ////////////////////////////////////////////////////////////////////////

    // ////////////////////////////////////////////////////////////////////////
    // Function subgroup: hilly terrain (11 noise functions)
    //
    // This subgroup generates the hilly terrain.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Hills module]: This billow-noise function generates the hills.
    let hilly_terrain_bi = Billow::<Perlin>::new(CURRENT_SEED + 60)
        .set_frequency(1663.0)
        .set_persistence(0.5)
        .set_lacunarity(HILLS_LACUNARITY)
        .set_octaves(6);

    // 2: [Scaled-hills module]: Next, a scale/bias module scales the output
    // value from the hills module so that its hilltops are not too high. The
    // reason for this is that these hills are eventually added to the river
    // valleys (see below).
    let hilly_terrain_sb0 = ScaleBias::new(hilly_terrain_bi)
        .set_scale(0.5)
        .set_bias(0.5);

    // 3: [River-valley module]: This ridged-multifractal-noise function generates
    // the river valleys. It has a much lower frequency so that more hills will
    // appear in between the valleys. Note that this noise function generates
    // ridged-multifractal noise using only one octave; this information will be
    // important in the next step.
    let hilly_terrain_rm = RidgedMulti::<Perlin>::new(CURRENT_SEED + 61)
        .set_frequency(367.5)
        .set_lacunarity(HILLS_LACUNARITY)
        .set_octaves(1);

    // 4: [Scaled-river-valley module]: Next, a scale/bias module applies a
    // scaling factor of -2.0 to the output value from the river-valley module.
    // This stretches the possible elevation values because one-octave ridged-
    // multifractal noise has a lower range of output values than multiple-
    // octave ridged-multifractal noise. The negative scaling factor inverts
    // the range of the output value, turning the ridges from the river-valley
    // module into valleys.
    let hilly_terrain_sb1 = ScaleBias::new(hilly_terrain_rm)
        .set_scale(-2.0)
        .set_bias(-1.0);

    // 5: [Low-flat module]: This low constant value is used by step 6.
    let hilly_terrain_co = Constant::new(-1.0);

    // 6: [Mountains-and-valleys module]: This blender module merges the scaled-
    // hills module and the scaled-river-valley module together. It causes the
    // low-lying areas of the terrain to become smooth, and causes the high-
    // lying areas of the terrain to contain hills. To do this, it uses uses the
    // scaled-hills module as the control module, causing the low-flat module to
    // appear in the lower areas and causing the scaled-river-valley module to
    // appear in the higher areas.
    let hilly_terrain_bl = Blend::new(hilly_terrain_co, hilly_terrain_sb1, hilly_terrain_sb0);

    // 7: [Scaled-hills-and-valleys module]: This scale/bias module slightly
    // reduces the range of the output value from the hills-and-valleys
    // module, decreasing the heights of the hilltops.
    let hilly_terrain_sb2 = ScaleBias::new(hilly_terrain_bl)
        .set_scale(0.75)
        .set_bias(-0.25);

    // 8: [Increased-slope-hilly-terrain module]: To increase the hill slopes
    // at higher elevations, this exponential-curve module applies an
    // exponential curve to the output value the scaled-hills-and-valleys
    // module. This exponential-curve module expects the input value to range
    // from -1.0 to 1.0.
    let hilly_terrain_ex: Exponent<_, _, 2> = Exponent::new(hilly_terrain_sb2).set_exponent(1.375);

    // 9: [Coarse-turbulence module]: This turbulence module warps the output
    // value from the increased-slope-hilly-terrain module, adding some
    // coarse detail to it.
    let hilly_terrain_tu0 = Turbulence::<_, Perlin>::new(hilly_terrain_ex)
        .set_seed(CURRENT_SEED + 62)
        .set_frequency(1531.0)
        .set_power(1.0 / 16921.0 * HILLS_TWIST)
        .set_roughness(4);

    // 10: [Warped-hilly-terrain module]: This turbulence module warps the
    // output value from the coarse-turbulence module. This turbulence has a
    // higher frequency, but lower power, than the coarse-turbulence module,
    // adding some fine detail to it.
    let hilly_terrain_tu1 = Turbulence::<_, Perlin>::new(hilly_terrain_tu0)
        .set_seed(CURRENT_SEED + 63)
        .set_frequency(21617.0)
        .set_power(1.0 / 117529.0 * HILLS_TWIST)
        .set_roughness(6);

    // 11: [Hilly-terrain group]: ACaches the output value from the warped-hilly-
    // terrain module. This is the output value for the entire hilly-terrain
    // group.
    let hilly_terrain = ACache::new(hilly_terrain_tu1);

    let noise_map = PlaneMapBuilder::new_fn(|x| hilly_terrain.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("hilly_terrain.png"));

    hilly_terrain
}

pub fn plains_terrain() -> impl NoiseFn<f64, 2> + Clone {
    // ////////////////////////////////////////////////////////////////////////
    // Function group: plains terrain
    // ////////////////////////////////////////////////////////////////////////

    // ////////////////////////////////////////////////////////////////////////
    // Function subgroup: plains terrain (7 noise functions)
    //
    // This subgroup generates the plains terrain.
    //
    // Because this subgroup will eventually be flattened considerably, the
    // types and combinations of noise functions that generate the plains are not
    // really that important; they only need to "look" interesting.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Plains-basis-0 module]: This billow-noise function, along with the
    // plains-basis-1 module, produces the plains.
    let plains_terrain_bi0 = Billow::<Perlin>::new(CURRENT_SEED + 70)
        .set_frequency(1097.5)
        .set_persistence(0.5)
        .set_lacunarity(PLAINS_LACUNARITY)
        .set_octaves(8);

    // 2: [Positive-plains-basis-0 module]: This scale/bias module makes the
    // output value from the plains-basis-0 module positive since this output
    // value will be multiplied together with the positive-plains-basis-1
    // module.
    let plains_terrain_sb0 = ScaleBias::new(plains_terrain_bi0)
        .set_scale(0.5)
        .set_bias(0.5);

    // 3: [Plains-basis-1 module]: This billow-noise function, along with the
    // plains-basis-2 module, produces the plains.
    let plains_terrain_bi1 = Billow::<Perlin>::new(CURRENT_SEED + 71)
        .set_frequency(1097.5)
        .set_persistence(0.5)
        .set_lacunarity(PLAINS_LACUNARITY)
        .set_octaves(8);

    // 4: [Positive-plains-basis-1 module]: This scale/bias module makes the
    // output value from the plains-basis-1 module positive since this output
    // value will be multiplied together with the positive-plains-basis-0
    // module.
    let plains_terrain_sb1 = ScaleBias::new(plains_terrain_bi1)
        .set_scale(0.5)
        .set_bias(0.5);

    // 5: [Combined-plains-basis module]: This multiplication module combines
    // the two plains basis modules together.
    let plains_terrain_mu = Multiply::new(plains_terrain_sb0, plains_terrain_sb1);

    // 6: [Rescaled-plains-basis module]: This scale/bias module maps the output
    // value that ranges from 0.0 to 1.0 back to a value that ranges from
    // -1.0 to +1.0.
    let plains_terrain_sb2: ScaleBias<_, _, 2> = ScaleBias::new(plains_terrain_mu)
        .set_scale(2.0)
        .set_bias(-1.0);

    // 7: [Plains-terrain group]: ACaches the output value from the rescaled-
    // plains-basis module.  This is the output value for the entire plains-
    // terrain group.
    let plains_terrain = ACache::new(plains_terrain_sb2);

    let noise_map = PlaneMapBuilder::new_fn(|x| plains_terrain.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("plains_terrain.png"));

    plains_terrain
}

pub(crate) fn badlands_sand_terrain() -> impl NoiseFn<f64, 2> + Clone {
    // ////////////////////////////////////////////////////////////////////////
    // Function subgroup: badlands sand (6 noise functions)
    //
    // This subgroup generates the sandy terrain for the badlands.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Sand-dunes module]: This ridged-multifractal-noise function generates
    // sand dunes. This ridged-multifractal noise is generated with a single
    // octave, which makes very smooth dunes.
    let badlands_sand_rm = RidgedMulti::<Perlin>::new(CURRENT_SEED + 80)
        .set_frequency(6163.5)
        .set_lacunarity(BADLANDS_LACUNARITY)
        .set_octaves(1);

    // 2: [Scaled-sand-dunes module]: This scale/bias module shrinks the dune
    // heights by a small amount. This is necessary so that the subsequent
    // noise functions in this subgroup can add some detail to the dunes.
    let badlands_sand_sb0: ScaleBias<_, _, 2> = ScaleBias::new(badlands_sand_rm)
        .set_scale(0.875)
        .set_bias(0.0);

    // 3: [Dune-detail module]: This noise function uses Voronoi polygons to
    // generate the detail to add to the dunes. By enabling the distance
    // algorithm, small polygonal pits are generated; the edges of the pits
    // are joined to the edges of nearby pits.
    let badlands_sand_wo = AWorley::new(CURRENT_SEED + 81)
        .set_frequency(16183.25)
        .set_return_type(ReturnType::Distance);

    // 4: [Scaled-dune-detail module]: This scale/bias module shrinks the dune
    // details by a large amount. This is necessary so that the subsequent
    // noise functions in this subgroup can add this detail to the sand-dunes
    // module.
    let badlands_sand_sb1 = ScaleBias::new(badlands_sand_wo)
        .set_scale(0.25)
        .set_bias(0.25);

    // 5: [Dunes-with-detail module]: This addition module combines the scaled-
    // sand-dunes module with the scaled-dune-detail module.
    let badlands_sand_ad = Add::new(badlands_sand_sb0, badlands_sand_sb1);

    // 6: [Badlands-sand subgroup]: ACaches the output value from the dunes-with-
    // detail module.
    let badlands_sand = ACache::new(badlands_sand_ad);

    let noise_map = PlaneMapBuilder::new_fn(|x| badlands_sand.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("badlands_sand.png"));

    badlands_sand
}

pub fn badlands_cliffs_terrain() -> impl NoiseFn<f64, 2> + Clone {
    // ////////////////////////////////////////////////////////////////////////
    // Function subgroup: badlands cliffs (7 noise functions)
    //
    // This subgroup generates the cliffs for the badlands.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Cliff-basis module]: This Perlin-noise function generates some coherent
    // noise that will be used to generate the cliffs.
    let badlands_cliffs_fb = Fbm::<Perlin>::new(CURRENT_SEED + 90)
        .set_frequency(CONTINENT_FREQUENCY * 839.0)
        .set_persistence(0.5)
        .set_lacunarity(BADLANDS_LACUNARITY)
        .set_octaves(6);

    // 2: [Cliff-shaping module]: Next, this curve module applies a curve to
    // the output value from the cliff-basis module. This curve is initially
    // very shallow, but then its slope increases sharply. At the highest
    // elevations, the curve becomes very flat again. This produces the
    // stereotypical Utah-style desert cliffs.
    let badlands_cliffs_cu = Curve::new(badlands_cliffs_fb)
        .add_control_point(-2.000, -2.000)
        .add_control_point(-1.000, -1.000)
        .add_control_point(-0.000, -0.750)
        .add_control_point(0.500, -0.250)
        .add_control_point(0.625, 0.875)
        .add_control_point(0.750, 1.000)
        .add_control_point(2.000, 1.250);

    // 3: [Clamped-cliffs module]: This clamping module makes the tops of the
    // cliffs very flat by clamping the output value from the cliff-shaping
    // module.
    let badlands_cliffs_cl: Clamp<_, _, 2> =
        Clamp::new(badlands_cliffs_cu).set_bounds(-999.125, 0.875);

    // 4: [Terraced-cliffs module]: Next, this terracing module applies some
    // terraces to the clamped-cliffs module in the lower elevations before the
    // sharp cliff transition.
    let badlands_cliffs_te = Terrace::new(badlands_cliffs_cl)
        .add_control_point(-1.000)
        .add_control_point(-0.875)
        .add_control_point(-0.750)
        .add_control_point(-0.500)
        .add_control_point(0.000)
        .add_control_point(1.000);

    // 5: [Coarse-turbulence module]: This turbulence module warps the output
    // value from the terraced-cliffs module, adding some coarse detail to it.
    let badlands_cliffs_tu0 = Turbulence::<_, Perlin>::new(badlands_cliffs_te)
        .set_seed(CURRENT_SEED + 91)
        .set_frequency(16111.0)
        .set_power(1.0 / 141539.0 * BADLANDS_TWIST)
        .set_roughness(3);

    // 6: [Warped-cliffs module]: This turbulence module warps the output value
    // from the coarse-turbulence module. This turbulence has a higher
    // frequency, but lower power, than the coarse-turbulence module, adding
    // some fine detail to it.
    let badlands_cliffs_tu1 = Turbulence::<_, Perlin>::new(badlands_cliffs_tu0)
        .set_seed(CURRENT_SEED + 92)
        .set_frequency(36107.0)
        .set_power(1.0 / 211543.0 * BADLANDS_TWIST)
        .set_roughness(3);

    // 7: [Badlands-cliffs subgroup]: ACaches the output value from the warped-
    // cliffs module.
    let badlands_cliffs = ACache::new(badlands_cliffs_tu1);

    let noise_map = PlaneMapBuilder::new_fn(|x| badlands_cliffs.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("badlands_cliffs.png"));

    badlands_cliffs
}

pub fn badlands_terrain(
    badlands_sand: ArcNoise<f64, 2>,
    badlands_cliffs: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // ////////////////////////////////////////////////////////////////////////
    // Function subgroup: badlands terrain (3 noise functions)
    //
    // Generates the final badlands terrain.
    //
    // Using a scale/bias module, the badlands sand is flattened considerably,
    // then the sand elevations are lowered to around -1.0. The maximum value
    // from the flattened sand module and the cliff module contributes to the
    // final elevation. This causes sand to appear at the low elevations since
    // the sand is slightly higher than the cliff base.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Scaled-sand-dunes module]: This scale/bias module considerably
    // flattens the output value from the badlands-sands subgroup and lowers
    // this value to near -1.0.
    let badlands_terrain_sb = ScaleBias::new(badlands_sand)
        .set_scale(0.25)
        .set_bias(-0.75);

    // 2: [Dunes-and-cliffs module]: This maximum-value module causes the dunes
    // to appear in the low areas and the cliffs to appear in the high areas.
    // It does this by selecting the maximum of the output values from the
    // scaled-sand-dunes module and the badlands-cliffs subgroup.
    let badlands_terrain_ma = Max::new(badlands_cliffs, badlands_terrain_sb);

    // 3: [Badlands-terrain group]: ACaches the output value from the dunes-and-
    // cliffs module. This is the output value for the entire badlands-terrain
    // group.
    let badlands_terrain = ACache::new(badlands_terrain_ma);

    //    debug::render_noise_module("complexplanet_images/12_2_badlandsTerrain.png",
    //                               &badlandsTerrain,
    //                               1024,
    //                               1024,
    //                               1000);

    let noise_map = PlaneMapBuilder::new_fn(|x| badlands_terrain.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("badlands_terrain.png"));

    badlands_terrain
}

pub fn river_positions() -> impl NoiseFn<f64, 2> + Clone {
    // ////////////////////////////////////////////////////////////////////////
    // Function subgroup: river positions (7 noise functions)
    //
    // This subgroup generates the river positions.
    //
    // -1.0 represents the lowest elevations and +1.0 represents the highest
    // elevations.
    //

    // 1: [Large-river-basis module]: This ridged-multifractal-noise function
    // creates the large, deep rivers.
    let river_positions_rm0 = RidgedMulti::<Perlin>::new(CURRENT_SEED + 100)
        .set_frequency(18.75)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(1);

    // 2: [Large-river-curve module]: This curve module applies a curve to the
    // output value from the large-river-basis module so that the ridges become
    // inverted. This creates the rivers. This curve also compresses the edge of
    // the rivers, producing a sharp transition from the land to the river
    // bottom.
    let river_positions_cu0 = Curve::new(river_positions_rm0)
        .add_control_point(-2.000, 2.000)
        .add_control_point(-1.000, 1.000)
        .add_control_point(-0.125, 0.875)
        .add_control_point(0.000, -1.000)
        .add_control_point(1.000, -1.500)
        .add_control_point(2.000, -2.000);

    // 3: [Small-river-basis module]: This ridged-multifractal-noise function
    // creates the small, shallow rivers.
    let river_positions_rm1 = RidgedMulti::<Perlin>::new(CURRENT_SEED + 101)
        .set_frequency(43.25)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(1);

    // 4: [Small-river-curve module]: This curve module applies a curve to the
    // output value from the small-river-basis module so that the ridges become
    // inverted. This creates the rivers. This curve also compresses the edge of
    // the rivers, producing a sharp transition from the land to the river
    // bottom.
    let river_positions_cu1 = Curve::new(river_positions_rm1)
        .add_control_point(-2.000, 2.0000)
        .add_control_point(-1.000, 1.5000)
        .add_control_point(-0.125, 1.4375)
        .add_control_point(0.000, 0.5000)
        .add_control_point(1.000, 0.2500)
        .add_control_point(2.000, 0.0000);

    // 5: [Combined-rivers module]: This minimum-value module causes the small
    // rivers to cut into the large rivers.  It does this by selecting the
    // minimum output values from the large-river-curve module and the small-
    // river-curve module.
    let river_positions_mi: Min<_, _, _, 2> = Min::new(river_positions_cu0, river_positions_cu1);

    // 6: [Warped-rivers module]: This turbulence module warps the output value
    //    from the combined-rivers module, which twists the rivers.  The high
    //    roughness produces less-smooth rivers.
    let river_positions_tu = Turbulence::<_, Perlin>::new(river_positions_mi)
        .set_seed(CURRENT_SEED + 102)
        .set_frequency(9.25)
        .set_power(1.0 / 57.75)
        .set_roughness(6);

    // 7: [River-positions group]: ACaches the output value from the warped-
    //    rivers module.  This is the output value for the entire river-
    //    positions group.
    let river_positions = ACache::new(river_positions_tu);

    let noise_map = PlaneMapBuilder::new_fn(|x| river_positions.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("river_positions.png"));

    river_positions
}

pub fn scaled_mountainous_terrain(
    mountainous_terrain: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: scaled mountainous terrain (6 noise functions)
    //
    // This subgroup scales the output value from the mountainous-terrain group
    // so that it can be added to the elevation defined by the continent-
    // definition group.
    //
    // This subgroup scales the output value such that it is almost always
    // positive.  This is done so that a negative elevation does not get applied
    // to the continent-definition group, preventing parts of that group from
    // having negative terrain features "stamped" into it.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Base-scaled-mountainous-terrain module]: This scale/bias module
    // scales the output value from the mountainous-terrain group so that the
    // output value is measured in planetary elevation units.
    let scaled_mountainous_terrain_sb0 = ScaleBias::new(mountainous_terrain)
        .set_scale(0.125)
        .set_bias(0.125);

    // 2: [Base-peak-modulation module]: At this stage, most mountain peaks have
    // roughly the same elevation. This BasicMulti module generates some
    // random values that will be used by subsequent noise functions to randomly
    // change the elevations of the mountain peaks.
    let scaled_mountainous_terrain_fb = Fbm::<Perlin>::new(CURRENT_SEED + 110)
        .set_frequency(14.5)
        .set_persistence(0.5)
        .set_lacunarity(MOUNTAIN_LACUNARITY)
        .set_octaves(6);

    // 3: [Peak-modulation module]: This exponential-curve module applies an
    // exponential curve to the output value from the base-peak-modulation
    // module. This produces a small number of high values and a much larger
    // number of low values. This means there will be a few peaks with much
    // higher elevations than the majority of the peaks, making the terrain
    // features more varied.
    let scaled_mountainous_terrain_ex =
        Exponent::new(scaled_mountainous_terrain_fb).set_exponent(1.25);

    // 4: [Scaled-peak-modulation module]: This scale/bias module modifies the
    // range of the output value from the peak-modulation module so that it can
    // be used as the modulator for the peak-height-multiplier module. It is
    // important that this output value is not much lower than 1.0.
    let scaled_mountainous_terrain_sb1 = ScaleBias::new(scaled_mountainous_terrain_ex)
        .set_scale(0.25)
        .set_bias(1.0);

    // 5: [Peak-height-multiplier module]: This multiplier module modulates the
    // heights of the mountain peaks from the base-scaled-mountainous-terrain
    // module using the output value from the scaled-peak-modulation module.
    let scaled_mountainous_terrain_mu = Multiply::new(
        scaled_mountainous_terrain_sb0,
        scaled_mountainous_terrain_sb1,
    );

    // 6: [Scaled-mountainous-terrain group]: ACaches the output value from the
    // peak-height-multiplier module.  This is the output value for the
    // entire scaled-mountainous-terrain group.
    let scaled_mountainous_terrain = ACache::new(scaled_mountainous_terrain_mu);

    let noise_map = PlaneMapBuilder::new_fn(|x| scaled_mountainous_terrain.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("scaled_mountainous_terrain.png"));

    scaled_mountainous_terrain
}

pub fn scaled_hilly_terrain(hilly_terrain: ArcNoise<f64, 2>) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: scaled hilly terrain (6 noise functions)
    //
    // This subgroup scales the output value from the hilly-terrain group so
    // that it can be added to the elevation defined by the continent-
    // definition group. The scaling amount applied to the hills is one half of
    // the scaling amount applied to the scaled-mountainous-terrain group.
    //
    // This subgroup scales the output value such that it is almost always
    // positive. This is done so that negative elevations are not applied to
    // the continent-definition group, preventing parts of the continent-
    // definition group from having negative terrain features "stamped" into it.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Base-scaled-hilly-terrain module]: This scale/bias module scales the
    // output value from the hilly-terrain group so that this output value is
    // measured in planetary elevation units.
    let scaled_hilly_terrain_sb0 = ScaleBias::new(hilly_terrain)
        .set_scale(0.0625)
        .set_bias(0.0625);

    // 2: [Base-hilltop-modulation module]: At this stage, most hilltops have
    // roughly the same elevation. This BasicMulti module generates some
    // random values that will be used by subsequent noise functions to
    // randomly change the elevations of the hilltops.
    let scaled_hilly_terrain_fb = Fbm::<Perlin>::new(CURRENT_SEED + 120)
        .set_frequency(13.5)
        .set_persistence(0.5)
        .set_lacunarity(HILLS_LACUNARITY)
        .set_octaves(6);

    // 3: [Hilltop-modulation module]: This exponential-curve module applies an
    // exponential curve to the output value from the base-hilltop-modulation
    // module. This produces a small number of high values and a much larger
    // number of low values. This means there will be a few hilltops with
    // much higher elevations than the majority of the hilltops, making the
    // terrain features more varied.
    let scaled_hilly_terrain_ex = Exponent::new(scaled_hilly_terrain_fb).set_exponent(1.25);

    // 4: [Scaled-hilltop-modulation module]: This scale/bias module modifies
    // the range of the output value from the hilltop-modulation module so that
    // it can be used as the modulator for the hilltop-height-multiplier module.
    // It is important that this output value is not much lower than 1.0.
    let scaled_hilly_terrain_sb1 = ScaleBias::new(scaled_hilly_terrain_ex)
        .set_scale(0.5)
        .set_bias(1.5);

    // 5: [Hilltop-height-multiplier module]: This multiplier module modulates
    // the heights of the hilltops from the base-scaled-hilly-terrain module
    // using the output value from the scaled-hilltop-modulation module.
    let scaled_hilly_terrain_mu = Multiply::new(scaled_hilly_terrain_sb0, scaled_hilly_terrain_sb1);

    // 6: [Scaled-hilly-terrain group]: ACaches the output value from the
    // hilltop-height-multiplier module. This is the output value for the entire
    // scaled-hilly-terrain group.
    let scaled_hilly_terrain = ACache::new(scaled_hilly_terrain_mu);

    let noise_map = PlaneMapBuilder::new_fn(|x| scaled_hilly_terrain.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("scaled_hilly_terrain.png"));

    scaled_hilly_terrain
}

pub fn scaled_plains_terrain(plains_terrain: ArcNoise<f64, 2>) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: scaled plains terrain (2 noise functions)
    //
    // This subgroup scales the output value from the plains-terrain group so
    // that it can be added to the elevations defined by the continent-
    // definition group.
    //
    // This subgroup scales the output value such that it is almost always
    // positive. This is done so that negative elevations are not applied to
    // the continent-definition group, preventing parts of the continent-
    // definition group from having negative terrain features "stamped" into it.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Scaled-plains-terrain module]: This scale/bias module greatly
    // flattens the output value from the plains terrain.  This output value
    // is measured in planetary elevation units.
    let scaled_plains_terrain_sb0 = ScaleBias::new(plains_terrain)
        .set_scale(0.00390625)
        .set_bias(0.0078125);

    // 2: [Scaled-plains-terrain group]: ACaches the output value from the
    // scaled-plains-terrain module. This is the output value for the entire
    // scaled-plains-terrain group.
    let scaled_plains_terrain = ACache::new(scaled_plains_terrain_sb0);

    let noise_map = PlaneMapBuilder::new_fn(|x| scaled_plains_terrain.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("scaled_plains_terrain.png"));

    scaled_plains_terrain
}

pub fn scaled_badlands_terrain(badlands_terrain: ArcNoise<f64, 2>) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: scaled badlands terrain (2 noise functions)
    //
    // This subgroup scales the output value from the badlands-terrain group so
    // that it can be added to the elevations defined by the continent-
    // definition group.
    //
    // This subgroup scales the output value such that it is almost always
    // positive. This is done so that negative elevations are not applied to the
    // continent-definition group, preventing parts of the continent-definition
    // group from having negative terrain features "stamped" into it.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Scaled-badlands-terrain module]: This scale/bias module scales the
    // output value from the badlands-terrain group so that it is measured
    // in planetary elevation units.
    let scaled_badlands_terrain_sb = ScaleBias::new(badlands_terrain)
        .set_scale(0.0625)
        .set_bias(0.0625);

    // 2: [Scaled-badlands-terrain group]: ACaches the output value from the
    // scaled-badlands-terrain module. This is the output value for the
    // entire scaled-badlands-terrain group.
    let scaled_badlands_terrain = ACache::new(scaled_badlands_terrain_sb);

    //    debug::render_noise_module("complexplanet_images/17_0_scaledBadlandsTerrain\
    //    .png",
    //                               &scaledBadlandsTerrain,
    //                               1024,
    //                               1024,
    //                               1000);

    let noise_map = PlaneMapBuilder::new_fn(|x| scaled_badlands_terrain.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("scaled_badlands_terrain.png"));

    scaled_badlands_terrain
}

pub fn continental_shelf(continent_def: ArcNoise<f64, 2>) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: continental shelf (6 noise functions)
    //
    // This module subgroup creates the continental shelves.
    //
    // The output value from this module subgroup are measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Shelf-creator module]: This terracing module applies a terracing
    // curve to the continent-definition group at the specified shelf level.
    // This terrace becomes the continental shelf. Note that this terracing
    // module also places another terrace below the continental shelf near -1.0.
    // The bottom of this terrace is defined as the bottom of the ocean;
    // subsequent noise functions will later add oceanic trenches to the bottom of
    // the ocean.
    let continental_shelf_te = Terrace::new(continent_def.clone())
        .add_control_point(-1.0)
        .add_control_point(-0.75)
        .add_control_point(SHELF_LEVEL)
        .add_control_point(1.0);

    //    debug::render_noise_module("complexplanet_images/18_0_continentalShelf_te\
    //    .png",
    //                               &continentalShelf_te,
    //                               1024,
    //                               1024,
    //                               1000);

    // 2: [Clamped-sea-bottom module]: This clamping module clamps the output
    // value from the shelf-creator module so that its possible range is from
    // the bottom of the ocean to sea level. This is done because this subgroup
    // is only concerned about the oceans.
    let continental_shelf_cl = Clamp::new(continental_shelf_te).set_bounds(-0.75, SEA_LEVEL);

    //    debug::render_noise_module("complexplanet_images/18_1_continentalShelf_cl\
    //    .png",
    //                               &continentalShelf_cl,
    //                               1024,
    //                               1024,
    //                               1000);

    // 3: [Oceanic-trench-basis module]: This ridged-multifractal-noise function
    // generates some coherent noise that will be used to generate the oceanic
    // trenches. The ridges represent the bottom of the trenches.
    let continental_shelf_rm = RidgedMulti::<Perlin>::new(CURRENT_SEED + 130)
        .set_frequency(CONTINENT_FREQUENCY * 4.375)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(16);

    //    debug::render_noise_module("complexplanet_images/18_2_continentalShelf_rm\
    //    .png",
    //                               &continentalShelf_rm,
    //                               1024,
    //                               1024,
    //                               1000);

    // 4: [Oceanic-trench module]: This scale/bias module inverts the ridges
    // from the oceanic-trench-basis-module so that the ridges become trenches.
    // This noise function also reduces the depth of the trenches so that their
    // depths are measured in planetary elevation units.
    let continental_shelf_sb = ScaleBias::new(continental_shelf_rm)
        .set_scale(-0.125)
        .set_bias(-0.125);

    //    debug::render_noise_module("complexplanet_images/18_3_continentalShelf_sb\
    //    .png",
    //                               &continentalShelf_sb,
    //                               1024,
    //                               1024,
    //                               1000);

    // 5: [Shelf-and-trenches module]: This addition module adds the oceanic
    // trenches to the clamped-sea-bottom module.
    let continental_shelf_ad = Add::new(continental_shelf_sb, continental_shelf_cl);

    // 6: [Continental-shelf subgroup]: ACaches the output value from the shelf-
    //    and-trenches module.
    let continental_shelf = ACache::new(continental_shelf_ad);

    //    debug::render_noise_module("complexplanet_images/18_4_continentalShelf.png",
    //                               &continentalShelf,
    //                               1024,
    //                               1024,
    //                               1000);

    let noise_map = PlaneMapBuilder::new_fn(|x| continental_shelf.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("continental_shelf.png"));

    continental_shelf
}

pub fn base_continent_elevations(
    continent_def: ArcNoise<f64, 2>,
    continental_shelf: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function group: base continent elevations (3 noise functions)
    //
    // This subgroup generates the base elevations for the continents, before
    // terrain features are added.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Base-scaled-continent-elevations module]: This scale/bias module
    // scales the output value from the continent-definition group so that it
    // is measured in planetary elevation units.
    let base_continent_elev_sb = ScaleBias::new(continent_def.clone())
        .set_scale(CONTINENT_HEIGHT_SCALE)
        .set_bias(0.0);

    //    debug::render_noise_module("complexplanet_images/19_0_baseContinentElev_sb\
    //    .png",
    //                               &baseContinentElev_sb,
    //                               1024,
    //                               1024,
    //                               1000);

    // 2: [Base-continent-with-oceans module]: This selector module applies the
    // elevations of the continental shelves to the base elevations of the
    // continent. It does this by selecting the output value from the
    // continental-shelf subgroup if the corresponding output value from the
    // continent-definition group is below the shelf level. Otherwise, it
    // selects the output value from the base-scaled-continent-elevations
    // module.
    let base_continent_elev_se = Select::new(
        base_continent_elev_sb,
        continental_shelf,
        continent_def.clone(),
    )
    .set_bounds(SHELF_LEVEL - 1000.0, SHELF_LEVEL)
    .set_falloff(0.03125);

    // 3: [Base-continent-elevation subgroup]: ACaches the output value from the
    // base-continent-with-oceans module.
    let base_continent_elev = ACache::new(base_continent_elev_se);

    //    debug::render_noise_module("complexplanet_images/19_1_baseContinentElev\
    //    .png",
    //                               &baseContinentElev,
    //                               1024,
    //                               1024,
    //                               1000);

    let noise_map = PlaneMapBuilder::new_fn(|x| base_continent_elev.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("base_continent_elev.png"));

    base_continent_elev
}

pub fn continents_with_plains(
    base_continent_elev: ArcNoise<f64, 2>,
    scaled_plains_terrain: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: continents with plains (2 noise functions)
    //
    // This subgroup applies the scaled-plains-terrain group to the base-
    // continent-elevation subgroup.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Continents-with-plains module]: This addition module adds the scaled-
    // plains-terrain group to the base-continent-elevation subgroup.
    let continents_with_plains_ad = Add::new(base_continent_elev.clone(), scaled_plains_terrain);

    // 2: [Continents-with-plains subgroup]: ACaches the output value from the
    // continents-with-plains module.
    let continents_with_plains = ACache::new(continents_with_plains_ad);

    //    debug::render_noise_module("complexplanet_images/20_0_continentsWithPlains\
    //    .png",
    //                               &continentsWithPlains,
    //                               1024,
    //                               1024,
    //                               1000);

    let noise_map = PlaneMapBuilder::new_fn(|x| continents_with_plains.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("continents_with_plains.png"));

    continents_with_plains
}

pub fn continents_with_hills(
    base_continent_elev: ArcNoise<f64, 2>,
    scaled_hilly_terrain: ArcNoise<f64, 2>,
    terrain_type_def: ArcNoise<f64, 2>,
    continents_with_plains: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: continents with hills (3 noise functions)
    //
    // This subgroup applies the scaled-hilly-terrain group to the continents-
    // with-plains subgroup.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Continents-with-hills module]: This addition module adds the scaled-
    // hilly-terrain group to the base-continent-elevation subgroup.
    let continents_with_hills_ad = Add::new(base_continent_elev.clone(), scaled_hilly_terrain);

    //    debug::render_noise_module("complexplanet_images/21_0_continentsWithHills_ad.png",
    //                               &continentsWithHills_ad,
    //                               1024,
    //                               1024,
    //                               1000);

    // 2: [Select-high-elevations module]: This selector module ensures that the
    // hills only appear at higher elevations. It does this by selecting the
    // output value from the continent-with-hills module if the corresponding
    // output value from the terrain-type-definition group is above a certain
    // value. Otherwise, it selects the output value from the continents-with-
    // plains subgroup.
    let continents_with_hills_se = Select::new(
        continents_with_plains,
        continents_with_hills_ad,
        terrain_type_def.clone(),
    )
    .set_bounds(1.0 - HILLS_AMOUNT, 1001.0 - HILLS_AMOUNT)
    .set_falloff(0.25);

    // 3: [Continents-with-hills subgroup]: ACaches the output value from the
    // select-high-elevations module.
    let continents_with_hills = ACache::new(continents_with_hills_se);

    //    debug::render_noise_module("complexplanet_images/21_1_continentsWithHills\
    //    .png",
    //                               &continentsWithHills,
    //                               1024,
    //                               1024,
    //                               1000);

    let noise_map = PlaneMapBuilder::new_fn(|x| continents_with_hills.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("continents_with_hills.png"));

    continents_with_hills
}

pub fn continents_with_mountains(
    base_continent_elev: ArcNoise<f64, 2>,
    scaled_mountainous_terrain: ArcNoise<f64, 2>,
    continent_def: ArcNoise<f64, 2>,
    terrain_type_def: ArcNoise<f64, 2>,
    continents_with_hills: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: continents with mountains (5 noise functions)
    //
    // This subgroup applies the scaled-mountainous-terrain group to the
    // continents-with-hills subgroup.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Continents-and-mountains module]: This addition module adds the
    // scaled-mountainous-terrain group to the base-continent-elevation
    // subgroup.
    let continents_with_mountains_ad0 =
        Add::new(base_continent_elev.clone(), scaled_mountainous_terrain);

    //    debug::render_noise_module("complexplanet_images/22_0_continentsWithMountains_ad0.png",
    //                               &continentsWithMountains_ad0,
    //                               1024,
    //                               1024,
    //                               1000);

    // 2: [Increase-mountain-heights module]: This curve module applies a curve
    // to the output value from the continent-definition group. This modified
    // output value is used by a subsequent noise function to add additional
    // height to the mountains based on the current continent elevation. The
    // higher the continent elevation, the higher the mountains.
    let continents_with_mountains_cu = Curve::new(continent_def)
        .add_control_point(-1.0, -0.0625)
        .add_control_point(0.0, 0.0000)
        .add_control_point(1.0 - MOUNTAINS_AMOUNT, 0.0625)
        .add_control_point(1.0, 0.2500);

    //    debug::render_noise_module("complexplanet_images/22_1_continentsWithMountains_cu.png",
    //                               &continentsWithMountains_cu,
    //                               1024,
    //                               1024,
    //                               1000);

    // 3: [Add-increased-mountain-heights module]: This addition module adds the
    // increased-mountain-heights module to the continents-and-mountains module.
    // The highest continent elevations now have the highest mountains.
    let continents_with_mountains_ad1 =
        Add::new(continents_with_mountains_ad0, continents_with_mountains_cu);

    //    debug::render_noise_module("complexplanet_images/22_2_continentsWithMountains_ad1.png",
    //                               &continentsWithMountains_ad1,
    //                               1024,
    //                               1024,
    //                               1000);

    // 4: [Select-high-elevations module]: This selector module ensures that
    // mountains only appear at higher elevations. It does this by selecting the
    // output value from the continent-with-mountains module if the
    // corresponding output value from the terrain-type-definition group is
    // above a certain value. Otherwise, it selects the output value from the
    // continents-with-hills subgroup. Note that the continents-with-hills
    // subgroup also contains the plains terrain.
    let continents_with_mountains_se = Select::new(
        continents_with_hills,
        continents_with_mountains_ad1,
        terrain_type_def.clone(),
    )
    .set_bounds(1.0 - MOUNTAINS_AMOUNT, 1001.0 - MOUNTAINS_AMOUNT)
    .set_falloff(0.25);

    // 5: [Continents-with-mountains subgroup]: ACaches the output value from the
    // select-high-elevations module.
    let continents_with_mountains = ACache::new(continents_with_mountains_se);

    //    debug::render_noise_module("complexplanet_images/22_3_continentsWithMountains.png",
    //                               &continentsWithMountains,
    //                               1024,
    //                               1024,
    //                               1000);

    let noise_map = PlaneMapBuilder::new_fn(|x| continents_with_mountains.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("continents_with_mountains.png"));

    continents_with_mountains
}

pub fn continents_with_badlands(
    base_continent_elev: ArcNoise<f64, 2>,
    scaled_badlands_terrain: ArcNoise<f64, 2>,
    continents_with_mountains: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: continents with badlands (5 noise functions)
    //
    // This subgroup applies the scaled-badlands-terrain group to the
    // continents-with-mountains subgroup.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Badlands-positions module]: This BasicMulti module generates some
    // random noise, which is used by subsequent noise functions to specify the
    // locations of the badlands.
    let continents_with_badlands_bm = Fbm::<Perlin>::new(CURRENT_SEED + 140)
        .set_frequency(16.5)
        .set_persistence(0.5)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(2);

    //    debug::render_noise_module("complexplanet_images/23_0_continentsWithBadlands_bm.png",
    //                               &continentsWithBadlands_bm,
    //                               1024,
    //                               1024,
    //                               1000);

    // 2: [Continents-and-badlands module]:  This addition module adds the
    // scaled-badlands-terrain group to the base-continent-elevation
    // subgroup.
    let continents_with_badlands_ad =
        Add::new(base_continent_elev.clone(), scaled_badlands_terrain);

    //    debug::render_noise_module("complexplanet_images/23_1_continentsWithBadlands_ad.png",
    //                               &continentsWithBadlands_ad,
    //                               1024,
    //                               1024,
    //                               1000);

    // 3: [Select-badlands-positions module]: This selector module places
    // badlands at random spots on the continents based on the BasicMulti noise
    // generated by the badlands-positions module. To do this, it selects the
    // output value from the continents-and-badlands module if the corresponding
    // output value from the badlands-position module is greater than a
    // specified value. Otherwise, this selector module selects the output value
    // from the continents-with-mountains subgroup. There is also a wide
    // transition between these two noise functions so that the badlands can blend
    // into the rest of the terrain on the continents.
    let continents_with_badlands_se = Select::new(
        continents_with_mountains.clone(),
        continents_with_badlands_ad,
        continents_with_badlands_bm,
    )
    .set_bounds(1.0 - BADLANDS_AMOUNT, 1001.0 - BADLANDS_AMOUNT)
    .set_falloff(0.25);

    //    debug::render_noise_module("complexplanet_images/23_2_continentsWithBadlands_se.png",
    //                               &continentsWithBadlands_se,
    //                               1024,
    //                               1024,
    //                               1000);

    // 4: [Apply-badlands module]: This maximum-value module causes the badlands
    // to "poke out" from the rest of the terrain. It does this by ensuring
    // that only the maximum of the output values from the continents-with-
    // mountains subgroup and the select-badlands-positions modules contribute
    // to the output value of this subgroup. One side effect of this process is
    // that the badlands will not appear in mountainous terrain.
    let continents_with_badlands_ma =
        Max::new(continents_with_mountains, continents_with_badlands_se);

    // 5: [Continents-with-badlands subgroup]: ACaches the output value from the
    //    apply-badlands module.
    let continents_with_badlands = ACache::new(continents_with_badlands_ma);

    //    debug::render_noise_module("complexplanet_images/23_3_continentsWithBadlands.png",
    //                               &continentsWithBadlands,
    //                               1024,
    //                               1024,
    //                               1000);

    let noise_map = PlaneMapBuilder::new_fn(|x| continents_with_badlands.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("continents_with_badlands.png"));

    continents_with_badlands
}

pub fn continents_with_rivers(
    river_positions: ArcNoise<f64, 2>,
    continents_with_badlands: ArcNoise<f64, 2>,
) -> impl NoiseFn<f64, 2> + Clone {
    // /////////////////////////////////////////////////////////////////////////
    // Function subgroup: continents with rivers (4 noise functions)
    //
    // This subgroup applies the river-positions group to the continents-with-
    // badlands subgroup.
    //
    // The output value from this module subgroup is measured in planetary
    // elevation units (-1.0 for the lowest underwater trenches and +1.0 for the
    // highest mountain peaks.)
    //

    // 1: [Scaled-rivers module]: This scale/bias module scales the output value
    // from the river-positions group so that it is measured in planetary
    // elevation units and is negative; this is required for step 2.
    let continents_with_rivers_sb = ScaleBias::new(river_positions)
        .set_scale(RIVER_DEPTH / 2.0)
        .set_bias(-RIVER_DEPTH / 2.0);

    //    debug::render_noise_module("complexplanet_images/24_0_continentsWithRivers_sb.png",
    //                               &continentsWithRivers_sb,
    //                               1024,
    //                               1024,
    //                               1000);

    // 2: [Add-rivers-to-continents module]: This addition module adds the
    // rivers to the continents-with-badlands subgroup. Because the scaled-
    // rivers module only outputs a negative value, the scaled-rivers module
    // carves the rivers out of the terrain.
    let continents_with_rivers_ad =
        Add::new(continents_with_badlands.clone(), continents_with_rivers_sb);

    //    debug::render_noise_module("complexplanet_images/24_1_continentsWithRivers_ad.png",
    //                               &continentsWithRivers_ad,
    //                               1024,
    //                               1024,
    //                               1000);

    // 3: [Blended-rivers-to-continents module]: This selector module outputs
    // deep rivers near sea level and shallower rivers in higher terrain.  It
    // does this by selecting the output value from the continents-with-
    // badlands subgroup if the corresponding output value from the
    // continents-with-badlands subgroup is far from sea level.  Otherwise,
    // this selector module selects the output value from the add-rivers-to-
    // continents module.
    let continents_with_rivers_se = Select::new(
        continents_with_badlands.clone(),
        continents_with_rivers_ad,
        continents_with_badlands,
    )
    .set_bounds(SEA_LEVEL, CONTINENT_HEIGHT_SCALE + SEA_LEVEL)
    .set_falloff(CONTINENT_HEIGHT_SCALE - SEA_LEVEL);

    // 4: [Continents-with-rivers subgroup]: ACaches the output value from the
    // blended-rivers-to-continents module.
    let continents_with_rivers = ACache::new(continents_with_rivers_se);

    let noise_map = PlaneMapBuilder::new_fn(|x| continents_with_rivers.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("continents_with_rivers.png"));

    continents_with_rivers
}

pub fn reference() -> impl NoiseFn<f64, 2> + Clone {
    let base_continent_def = ArcNoise::new(base_continent_def());
    let continent_def = ArcNoise::new(continent_def(base_continent_def.clone()));
    let terrain_type_def = ArcNoise::new(terrain_type_def(continent_def.clone()));

    let mountainous_base_def = ArcNoise::new(mountainous_base_def());
    let high_mountainous_terrain = ArcNoise::new(high_mountainous_terrain());
    let low_mountainous_terrain = ArcNoise::new(low_mountainous_terrain());
    let mountainous_terrain = ArcNoise::new(mountainous_terrain(
        mountainous_base_def.clone(),
        high_mountainous_terrain.clone(),
        low_mountainous_terrain.clone(),
    ));
    let hilly_terrain = ArcNoise::new(hilly_terrain());
    let plains_terrain = ArcNoise::new(plains_terrain());

    let badlands_sand_terrain = ArcNoise::new(badlands_sand_terrain());
    let badlands_cliffs_terrain = ArcNoise::new(badlands_cliffs_terrain());
    let badlands_terrain = ArcNoise::new(badlands_terrain(
        badlands_sand_terrain.clone(),
        badlands_cliffs_terrain.clone(),
    ));

    let river_positions = ArcNoise::new(river_positions());
    let scaled_mountainous_terrain =
        ArcNoise::new(scaled_mountainous_terrain(mountainous_terrain.clone()));
    let scaled_hilly_terrain = ArcNoise::new(scaled_hilly_terrain(hilly_terrain.clone()));
    let scaled_plains_terrain = ArcNoise::new(scaled_plains_terrain(plains_terrain.clone()));
    let scaled_badlands_terrain = ArcNoise::new(scaled_badlands_terrain(badlands_terrain.clone()));

    let continental_shelf = ArcNoise::new(continental_shelf(continent_def.clone()));
    let base_continent_elevations = ArcNoise::new(base_continent_elevations(
        continent_def.clone(),
        continental_shelf.clone(),
    ));

    let continents_with_plains = ArcNoise::new(continents_with_plains(
        base_continent_elevations.clone(),
        scaled_plains_terrain.clone(),
    ));

    let continents_with_hills = ArcNoise::new(continents_with_hills(
        base_continent_elevations.clone(),
        scaled_hilly_terrain.clone(),
        terrain_type_def.clone(),
        continents_with_plains.clone(),
    ));

    let continents_with_mountains = ArcNoise::new(continents_with_mountains(
        base_continent_elevations.clone(),
        scaled_mountainous_terrain.clone(),
        continent_def.clone(),
        terrain_type_def.clone(),
        continents_with_hills.clone(),
    ));

    let continents_with_badlands = ArcNoise::new(continents_with_badlands(
        base_continent_elevations.clone(),
        scaled_badlands_terrain.clone(),
        continents_with_mountains.clone(),
    ));

    let continents_with_rivers = ArcNoise::new(continents_with_rivers(
        river_positions.clone(),
        continents_with_badlands.clone(),
    ));

    let unscaled_final_planet = ACache::new(continents_with_rivers);

    // exit(0);

    let noise_map = PlaneMapBuilder::new_fn(|x| unscaled_final_planet.get(x))
        .set_size(1024, 1024)
        // .set_x_bounds(-32768.0, 32768.0)
        // .set_y_bounds(-32768.0, 32768.0)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();
    noise_map.write_to_file(Path::new("planet.png"));

    unscaled_final_planet
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use noise::{core::worley::ReturnType, utils::PlaneMapBuilder, NoiseFn};

    use crate::isosurface::surface::csg::aworley::AWorley;

    #[test]
    fn workey() {
        let badlands_sand_wo = AWorley::new(32 + 81)
            .set_frequency(10.0)
            .set_return_type(ReturnType::Value);

        let noise_map = PlaneMapBuilder::new_fn(|x: [f64; 2]| badlands_sand_wo.get(x))
            .set_size(1024, 1024)
            // .set_x_bounds(-32768.0, 32768.0)
            // .set_y_bounds(-32768.0, 32768.0)
            .set_x_bounds(-2.0, 2.0)
            .set_y_bounds(-2.0, 2.0)
            .build();
        noise_map.write_to_file(Path::new("workey_test.png"));
    }
}
