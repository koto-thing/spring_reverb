#[derive(Clone)]
pub struct AllPassFilter {
    g: f32,     // フィルタ係数
    z_in: f32,  // 1サンプル前の入力(x[n - 1])
    z_out: f32, // 1サンプル前の出力(y[n - 1])
}

impl AllPassFilter {
    pub fn new(g: f32) -> Self {
        Self {
            g,
            z_in: 0.0,
            z_out: 0.0,
        }
    }
    
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        // Direct form I
        let output = self.g * input + self.z_in - self.g * self.z_out;
        
        // 状態を更新
        self.z_in = input;
        self.z_out = output;
        
        output
    }
}