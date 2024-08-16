/// white noise 随机噪声
#define_import_path noise::white

// copy from https://www.shadertoy.com/view/tlcBRl

// mini
fn white_noise_21_v1(seed1: f32, seed2: f32) -> f32 {
    let a = abs(seed1 * 0.91) + seed2 + 94.68;
    let c = fract((abs(seed2 * 0.41) + 45.46) * fract((abs(seed2) + 757.21) * fract(seed1 * 0.0171)));
    let b = fract(100.0 * a * c);
    return (fract(seed1 + 12.34567 * b)) * 1.0038 - 0.00185;
}

//2 seeds
fn white_noise_21_v2(seed1: f32, seed2: f32) -> f32 {
    float buff1 = abs(seed1 + 100.94) + 1000.;
    float buff2 = abs(seed2 + 100.73) + 1000.;
    buff1 = (buff1 * fract(buff2 * fract(buff1 * fract(buff2 * 0.63))));
    buff2 = (buff2 * fract(buff2 * fract(buff1 + buff2 * fract(seed1 * 0.79))));
    buff1 = white_noise_21_v1(buff1, buff2);
    return(buff1 * 1.0038 - 0.00185);
}

//3 seeds
fn white_noise_31_v1(seed1: f32, seed2: f32, seed3: f32) -> f32 {
    float buff1 = abs(seed1 + 100.81) + 1000.3;
    float buff2 = abs(seed2 + 100.45) + 1000.2;
    float buff3 = abs(white_noise_21_v1(seed1, seed2) + seed3) + 1000.1;
    buff1 = (buff3 * fract(buff2 * fract(buff1 * fract(buff2 * 0.146))));
    buff2 = (buff2 * fract(buff2 * fract(buff1 + buff2 * fract(buff3 * 0.52))));
    buff1 = white_noise_21_v1(buff1, buff2);
    return(buff1);
}

// 3 seeds hard
fn white_noise_31_v2(seed1: f32, seed2: f32, seed3: f32) -> f32 {
    float buff1 = abs(seed1 + 100.813) + 1000.314;
    float buff2 = abs(seed2 + 100.453) + 1000.213;
    float buff3 = abs(white_noise_21_v1(buff2, buff1) + seed3) + 1000.17;
    buff1 = (buff3 * fract(buff2 * fract(buff1 * fract(buff2 * 0.14619))));
    buff2 = (buff2 * fract(buff2 * fract(buff1 + buff2 * fract(buff3 * 0.5215))));
    buff1 = white_noise_31_v1(white_noise_21_v1(seed2, buff1), white_noise_21_v1(seed1, buff2), white_noise_21_v1(seed3, buff3));
    return(buff1);
}