# spring_reverb

## スプリングリーバーブIR生成器

* 物理モデリング
* アルゴリズム

の二つの方法からIRを生成しています。

### 物理モデリング

ばねの波動方程式

$\frac{\partial^2 y}{\partial t^2} = -k^2 \frac{\partial^4 y}{\partial x^4} - c \frac{\partial y}{\partial t}$

を用いて、数値的に解いてます。

具体的には、時間tと位置xのグリッドを定義して、テイラー展開を利用してばねの変位yを更新しています。  
変位yを更新する式をもとに、インパルスを入力し、必要なRTの分のサンプルを計算してIRを生成しています。

### アルゴリズム

* リングバッファ
* オールパスフィルタ
* ローパスフィルタ
* ハイパスフィルタ

を組み合わせて、ばねの特性を再現しています。

---

AI wrote the section below this point.

## Python setup for IR analysis

This project includes a small Python utility to inspect generated impulse responses (`ir_*.wav`).

### 1) Create and activate a virtual environment (PowerShell)

```powershell
Set-Location "D:\Pandd\spring_reverb"
python -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
pip install -r requirements.txt
```

### 2) Analyze one or more IR files

```powershell
python tools\analyze_ir.py ir_physical.wav
python tools\analyze_ir.py ir_physical.wav ir_algorithm.wav ir_hybrid.wav
```

Results are written to `analysis_out/` as PNG reports, and summary metrics are printed to the console:
- sample rate
- signal length
- peak / RMS
- DC offset
- clip ratio (`|x| >= 0.9999`)
- out-of-range ratio before clip (`|x| > 1`)
- non-finite ratio (`NaN`/`Inf`)
- RT60 estimate (Schroeder decay fit, if decay range is sufficient)

### 3) Optional output directory

```powershell
python tools\analyze_ir.py ir_physical.wav --output-dir analysis_out\physical_only
```

## Notes

- Input WAV can be mono or stereo (stereo is averaged to mono for analysis).
- If RT60 shows `n/a`, the IR may be too short or not decaying enough for a robust estimate.

