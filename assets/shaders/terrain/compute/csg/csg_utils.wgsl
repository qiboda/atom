#define_import_path terrain::csg::csg_utils

#import csg::csg_operate::{op_round, op_union, op_subtraction, op_intersection, op_smooth_union, op_smooth_subtraction, op_smooth_intersection, op_subtraction_exact}
#import csg::csg_shape::{sd_sphere, sd_box}

#import terrain::main_mesh_bind_group::csg_operations


fn apply_csg_operations(density_position: vec3f, density: f32) -> f32 {
    var result = density;

    for (var i = 0u; i < arrayLength(&csg_operations); i++) {
        let csg = csg_operations[i];

        let local_position = density_position - csg.position;

        var csg_value = 0.0;
        switch(csg.primitive_type) {
            case 0u {
                csg_value = sd_sphere(local_position, csg.shape.x);
            }
            case 1u {
                csg_value = sd_box(local_position, csg.shape.xyz);
            }
            default: {
            }
        }

        switch(csg.operate_type) {
            case 0u {
                result = op_round(csg_value, result);
            }
            case 1u {
                result = op_union(csg_value, result);
            }
            case 2u {
                // 非smooth版本有法线错误的问题。如果需要精确表示，需要修复。
                result = op_subtraction(csg_value, result);
            }
            case 3u {
                result = op_intersection(csg_value, result);
            }
            case 4u {
                result = op_smooth_union(csg_value, result, 0.3);
            }
            case 5u {
                // k的值，需要根据csg的shape的尺寸实际情况调整
                result = op_smooth_subtraction(csg_value,result, 0.3);
            }
            case 6u {
                result = op_smooth_intersection(csg_value, result, 0.3);
            }
            default: {
            }
        }
    }

    return result;
}