#!/usr/bin/env python3
"""Impulse response analysis helper for spring_reverb WAV outputs."""

from __future__ import annotations

import argparse
import math
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from scipy.io import wavfile
from scipy.signal import spectrogram


EPS = 1e-12
CLIP_THRESHOLD = 0.9999


def load_mono_wav(path: Path) -> tuple[int, np.ndarray, dict[str, float]]:
    sample_rate, data = wavfile.read(path)

    if np.issubdtype(data.dtype, np.integer):
        data = data.astype(np.float64) / np.iinfo(data.dtype).max
    else:
        with np.errstate(invalid="ignore"):
            data = data.astype(np.float64)

    if data.ndim == 2:
        data = data.mean(axis=1)

    total = float(data.size) if data.size else 1.0
    finite_mask = np.isfinite(data)
    non_finite_ratio = float(1.0 - (np.sum(finite_mask) / total))

    # Some exported WAVs can contain non-finite values; clamp them to silence.
    data = np.nan_to_num(data, nan=0.0, posinf=0.0, neginf=0.0)

    out_of_range_ratio = float(np.mean(np.abs(data) > 1.0)) if data.size else 0.0

    # Float WAV is expected near [-1, 1]. Clip outlier values from broken exports.
    data = np.clip(data, -1.0, 1.0)

    clip_ratio = float(np.mean(np.abs(data) >= CLIP_THRESHOLD)) if data.size else 0.0
    dc_offset = float(np.mean(data)) if data.size else 0.0

    quality = {
        "non_finite_ratio": non_finite_ratio,
        "out_of_range_ratio": out_of_range_ratio,
        "clip_ratio": clip_ratio,
        "dc_offset": dc_offset,
    }

    return int(sample_rate), data, quality


def schroeder_decay_db(ir: np.ndarray) -> np.ndarray:
    energy = ir * ir
    reverse_cumsum = np.cumsum(energy[::-1])[::-1]
    reverse_cumsum = np.maximum(reverse_cumsum, EPS)
    initial = max(float(reverse_cumsum[0]), EPS)
    decay_db = 10.0 * np.log10(reverse_cumsum / initial)
    return decay_db


def estimate_rt60(sample_rate: int, decay_db: np.ndarray) -> float | None:
    levels = [(-5.0, -35.0), (-5.0, -25.0), (-5.0, -15.0)]
    t = np.arange(decay_db.shape[0], dtype=np.float64) / sample_rate

    for hi, lo in levels:
        mask = (decay_db <= hi) & (decay_db >= lo)
        if int(np.sum(mask)) < 20:
            continue

        t_fit = t[mask]
        y_fit = decay_db[mask]
        slope, _ = np.polyfit(t_fit, y_fit, 1)
        if slope >= 0.0:
            continue

        rt = 60.0 / abs(slope)
        if math.isfinite(rt):
            return float(rt)

    return None


def make_report_figure(sample_rate: int, ir: np.ndarray, decay_db: np.ndarray, out_png: Path) -> None:
    t = np.arange(ir.shape[0], dtype=np.float64) / sample_rate

    f, bins, spec = spectrogram(
        ir,
        fs=sample_rate,
        nperseg=min(1024, max(128, ir.shape[0] // 8)),
        noverlap=None,
        scaling="spectrum",
        mode="magnitude",
    )

    spec_db = 20.0 * np.log10(np.maximum(spec, EPS))

    fig, axes = plt.subplots(3, 1, figsize=(10, 10), constrained_layout=True)

    axes[0].plot(t, ir, linewidth=0.8)
    axes[0].set_title("Waveform")
    axes[0].set_xlabel("Time [s]")
    axes[0].set_ylabel("Amplitude")
    axes[0].grid(True, alpha=0.25)

    axes[1].plot(t, decay_db, linewidth=1.0)
    axes[1].set_title("Schroeder Decay")
    axes[1].set_xlabel("Time [s]")
    axes[1].set_ylabel("Level [dB]")
    axes[1].set_ylim(-90, 3)
    axes[1].grid(True, alpha=0.25)

    mesh = axes[2].pcolormesh(bins, f, spec_db, shading="gouraud")
    axes[2].set_title("Spectrogram")
    axes[2].set_xlabel("Time [s]")
    axes[2].set_ylabel("Frequency [Hz]")
    fig.colorbar(mesh, ax=axes[2], label="Magnitude [dB]")

    out_png.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_png, dpi=150)
    plt.close(fig)


def analyze_file(path: Path, output_dir: Path) -> dict[str, float | int | str | None]:
    sample_rate, ir, quality = load_mono_wav(path)
    peak = float(np.max(np.abs(ir))) if ir.size else 0.0
    rms = float(np.sqrt(np.mean(ir * ir))) if ir.size else 0.0
    decay_db = schroeder_decay_db(ir)
    rt60 = estimate_rt60(sample_rate, decay_db)

    out_png = output_dir / f"{path.stem}_analysis.png"
    make_report_figure(sample_rate, ir, decay_db, out_png)

    return {
        "file": str(path),
        "sample_rate": sample_rate,
        "samples": int(ir.shape[0]),
        "length_sec": float(ir.shape[0] / sample_rate),
        "peak": peak,
        "rms": rms,
        "dc_offset": quality["dc_offset"],
        "clip_ratio": quality["clip_ratio"],
        "out_of_range_ratio": quality["out_of_range_ratio"],
        "non_finite_ratio": quality["non_finite_ratio"],
        "rt60_sec": rt60,
        "report_png": str(out_png),
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Analyze impulse response WAV files.")
    parser.add_argument(
        "inputs",
        nargs="+",
        type=Path,
        help="Input WAV file(s), e.g. ir_physical.wav ir_algorithm.wav",
    )
    parser.add_argument(
        "-o",
        "--output-dir",
        type=Path,
        default=Path("analysis_out"),
        help="Directory for analysis artifacts (default: analysis_out)",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    for input_path in args.inputs:
        if not input_path.exists():
            raise FileNotFoundError(f"Input not found: {input_path}")
        if input_path.suffix.lower() != ".wav":
            raise ValueError(f"Input must be a WAV file: {input_path}")

    print("IR analysis results")
    print("=" * 72)
    for input_path in args.inputs:
        result = analyze_file(input_path, args.output_dir)
        print(f"File       : {result['file']}")
        print(f"SampleRate : {result['sample_rate']} Hz")
        print(f"Length     : {result['length_sec']:.3f} s ({result['samples']} samples)")
        print(f"Peak       : {result['peak']:.6f}")
        print(f"RMS        : {result['rms']:.6f}")
        print(f"DC Offset  : {result['dc_offset']:.6f}")
        print(f"Clip Ratio : {100.0 * result['clip_ratio']:.2f} % (|x| >= {CLIP_THRESHOLD})")
        print(f"|x| > 1    : {100.0 * result['out_of_range_ratio']:.2f} % (before clip)")
        print(f"Non-finite : {100.0 * result['non_finite_ratio']:.2f} %")
        if result["rt60_sec"] is None:
            print("RT60       : n/a (insufficient decay range)")
        else:
            print(f"RT60       : {result['rt60_sec']:.3f} s")
        print(f"Report PNG : {result['report_png']}")
        print("-" * 72)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

