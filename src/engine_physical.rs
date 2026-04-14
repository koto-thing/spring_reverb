pub struct SpringFDTD {
    // 状態を保持するバッファ
    u_next: Vec<f32>,
    u_curr: Vec<f32>,
    u_past: Vec<f32>,

    // 物理パラメータ
    rho_sq:      f32, // ρ^2(張力係数)
    gamma_sq:    f32, // γ^2(剛性係数)
    sigma_theta: f32, // σθ(減衰係数)

    // 入出力位置
    pickup_pos: usize,
}

impl SpringFDTD {
    pub fn new(num_points: usize, rho: f32, gamma: f32, sigma: f32, sample_rate: f32) -> Self {
        let dt = 1.0 / sample_rate;
        let sigma_theta = sigma * dt;
        let rho_sq = rho * rho;
        let gamma_sq = gamma * gamma;

        let stability = rho_sq + 4.0 * gamma_sq;
        println!("--- FDTD Stability Check ---");
        println!("  rho: {}, gamma: {}", rho, gamma);
        println!("  rho^2 + 4*gamma^2 = {:.4}", stability);
        if stability <= 1.0 {
            println!("  Status: STABLE (<= 1.0) - OK");
        } else {
            println!("  Status: UNSTABLE (> 1.0) - WARNING");
        }
        println!("----------------------------");

        Self {
            u_next: vec![0.0; num_points],
            u_curr: vec![0.0; num_points],
            u_past: vec![0.0; num_points],
            rho_sq,
            gamma_sq,
            sigma_theta,
            pickup_pos: num_points / 3,
        }
    }

    /// インパルスを特定の位置に加える
    /// position: インパルスを加える位置
    /// force: インパルスの強さ
    pub fn add_impulse(&mut self, position: usize, force: f32) {
        if position > 1 && position < self.u_curr.len() - 2 {
            self.u_curr[position] += force;
            self.u_past[position] += force; // 速度0の初期変位を作る
        }
    }

    /// 1サンプル分のシミュレーションを進めて、出力位置の音を返す
    /// 返り値: 出力位置の変位
    pub fn step(&mut self) -> f32 {
        let n = self.u_curr.len();
        let inv_den = 1.0 / (1.0 + self.sigma_theta);
        let damp_past = 1.0 - self.sigma_theta;

        // 両端は固定端にしておく
        for i in 2..n - 2 {
            // 張力項を計算(2階の空間微分)
            let tension = self.rho_sq * (self.u_curr[i + 1] - 2.0 * self.u_curr[i] + self.u_curr[i - 1]);

            // 剛性項を計算(4階の空間微分)
            let stiffness = self.gamma_sq * (self.u_curr[i + 2] - 4.0 * self.u_curr[i + 1] + 6.0 * self.u_curr[i] - 4.0 * self.u_curr[i - 1] + self.u_curr[i - 2]);

            // 更新
            self.u_next[i] = inv_den * (
                2.0 * self.u_curr[i] - damp_past * self.u_past[i] + tension - stiffness
            );
        }

        // バッファをローテーションする
        std::mem::swap(&mut self.u_past, &mut self.u_curr);
        std::mem::swap(&mut self.u_curr, &mut self.u_next);

        // 出力位置の変位を返す
        self.u_curr[self.pickup_pos]
    }


}