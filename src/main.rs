use hound;
use crate::engine_algorithm::SpringAlgorithm;
use crate::engine_physical::SpringFDTD;

mod engine_physical;
mod engine_algorithm;
mod all_pass_filter;
mod delay_line;

const SAMPLE_RATE: u32 = 44100;
const IR_DURATION_SEC: f32 = 3.0;
const TARGET_RMS_DBFS: f32 = -55.0;

fn normalize(samples: &[f32]) -> Vec<f32> {
    // RMSを計算
    let rms = (samples.iter().map(|&sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt();
    if rms < 1e-9 {
        return samples.to_vec();
    }

    // 目標RMSに合わせるゲインを計算する
    let target_rms = 10_f32.powf(TARGET_RMS_DBFS / 20.0);
    let rms_gain = target_rms / rms;

    // RMSゲイン適用後のピークを確認
    let peak_after = samples.iter().cloned().map(|sample| (sample * rms_gain).abs()).fold(0.0f32, f32::max);

    // クリップ
    let final_gain = if peak_after > 0.99 {
        rms_gain * (0.99 / peak_after)
    } else {
        rms_gain
    };

    let rms_db = 20.0 * rms.log10();
    let gain_db = 20.0 * final_gain.log10();

    println!("    RMS: {:.2} dBFS  gain: {:+.2} dB  -> target: {:.2} dBFS",
             rms_db, gain_db, rms_db + gain_db);

    samples.iter().map(|&s| s * final_gain).collect()
}

fn write_wav(path: &str, samples: &[f32]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    let normalized = normalize(samples);
    for &s in &normalized {
        writer.write_sample(s.clamp(-1.0, 1.0)).unwrap();
    }

    writer.finalize().unwrap();
    println!("  -> {} ({} samples)", path, samples.len());
}

fn generate_ir_physical(duration_sec: f32) -> Vec<f32> {
    let num_samples = (SAMPLE_RATE as f32 * duration_sec) as usize;
    let mut engine = SpringFDTD::new(
        2000,
        0.45,
        0.35,
        3.5,
        SAMPLE_RATE as f32
    );

    engine.add_impulse(150, 0.01);
    (0..num_samples).map(|_| engine.step()).collect()
}

fn generate_ir_algorithm(duration_sec: f32) -> Vec<f32> {
    // 約31ms, 約37ms, 約43ms
    let num_samples = (SAMPLE_RATE as f32 * duration_sec) as usize;
    let mut spring1 = SpringAlgorithm::new((SAMPLE_RATE as f32 * 0.0313) as usize, 15, 0.65, 0.85);
    let mut spring2 = SpringAlgorithm::new((SAMPLE_RATE as f32 * 0.0371) as usize, 15, 0.62, 0.86);
    let mut spring3 = SpringAlgorithm::new((SAMPLE_RATE as f32 * 0.0433) as usize, 15, 0.59, 0.84);

    (0..num_samples).map(|i| {
        let input = if i == 0 { 1.0 } else { 0.0 };

        // 並列で入力して、まぜまぜ
        let out1 = spring1.process(input);
        let out2 = spring2.process(input);
        let out3 = spring3.process(input);

        (out1 + out2 + out3) / 3.0
    }).collect()
}

fn generate_ir_hybrid(duration_sec: f32) -> Vec<f32> {
    let fdtd_ir = generate_ir_physical(duration_sec);
    let mut algo = SpringAlgorithm::new(
        (SAMPLE_RATE as f32 * 0.025) as usize,
        4, 0.5, 0.7,
    );

    fdtd_ir.iter().map(|&s| algo.process(s * 0.005)).collect()
}

fn main() {
    println!("Spring Reverb IR Generator");
    println!("  Sample rate : {} Hz", SAMPLE_RATE);
    println!("  IR duration : {} sec", IR_DURATION_SEC);
    println!();

    println!("[1/3] Generating physical model IR...");
    let ir_physical = generate_ir_physical(IR_DURATION_SEC);
    write_wav("ir_physical.wav", &ir_physical);

    println!("[2/3] Generating algorithm IR...");
    let ir_algorithm = generate_ir_algorithm(IR_DURATION_SEC);
    write_wav("ir_algorithm.wav", &ir_algorithm);

    println!("[3/3] Generating hybrid IR...");
    let ir_hybrid = generate_ir_hybrid(IR_DURATION_SEC);
    write_wav("ir_hybrid.wav", &ir_hybrid);

    println!();
    println!("Done! 3 IR WAV files generated.");
}
