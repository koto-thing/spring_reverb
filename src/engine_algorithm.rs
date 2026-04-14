use crate::all_pass_filter::AllPassFilter;
use crate::delay_line::DelayLine;

pub struct SpringAlgorithm {
    delay:         DelayLine,
    apf_cascade:   Vec<AllPassFilter>,
    feedback_gain: f32, // 反射時のエネルギー損失

    lpf_state:  f32,
    lpf_cutoff: f32,

    hpf_state:   f32,
    hpf_prev_in: f32,
}

impl SpringAlgorithm {
    pub fn new(delay_samples: usize, num_apfs: usize, apf_g: f32, feedback: f32) -> Self {
        Self {
            delay: DelayLine::new(delay_samples),
            apf_cascade: vec![AllPassFilter::new(apf_g); num_apfs],
            feedback_gain: feedback,

            lpf_state: 0.0,
            lpf_cutoff: 0.4,

            hpf_state: 0.0,
            hpf_prev_in: 0.0,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        // フィードバックループの入力を計算する
        let mut loop_in = input + (self.lpf_state * self.feedback_gain);
        loop_in = loop_in.clamp(-1.0, 1.0);

        // 遅延線を通過させる
        let mut signal = self.delay.process(loop_in);

        // カスケードされたAPFを通過させる
        for apf in self.apf_cascade.iter_mut() {
            signal = apf.process(signal);
        }

        // ローパスフィルタを通過させる
        self.lpf_state += self.lpf_cutoff * (signal - self.lpf_state);
        signal = self.lpf_state;

        // ハイパスフィルタを通過させる
        let r = 0.995; // カットオフ周波数
        self.hpf_state = signal - self.hpf_prev_in + r * self.hpf_state;
        self.hpf_prev_in = signal;

        // クリップ
        self.hpf_state = (self.hpf_state * 1.5).tanh() / 1.5;

        self.hpf_state
    }
}