pub struct DelayLine {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayLine {
    pub fn new(delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; delay_samples],
            write_pos: 0,
        }
    }
    
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        // 現在の書き込み位置の音を読みだす
        let output = self.buffer[self.write_pos];
        
        // 新しい音を書き込む
        self.buffer[self.write_pos] = input;
        
        // インデックスを次の位置に進める
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        
        output
    }
}