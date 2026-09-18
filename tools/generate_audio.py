#!/usr/bin/env python3
"""
Procedural audio generator for fragr.
Generates simple WAV files using sine waves, square waves, and noise.
Output: client/assets/audio/*.wav
License: CC0 1.0 Universal (Public Domain)
"""

import wave
import struct
import math
import random
import os

SAMPLE_RATE = 22050
OUTPUT_DIR = "../client/assets/audio"


def generate_sine_wave(frequency, duration, amplitude=0.5):
    """Generate a sine wave."""
    num_samples = int(SAMPLE_RATE * duration)
    samples = []
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        value = amplitude * math.sin(2 * math.pi * frequency * t)
        samples.append(value)
    return samples


def generate_square_wave(frequency, duration, amplitude=0.3):
    """Generate a square wave."""
    num_samples = int(SAMPLE_RATE * duration)
    samples = []
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        value = amplitude if math.sin(2 * math.pi * frequency * t) > 0 else -amplitude
        samples.append(value)
    return samples


def generate_noise(duration, amplitude=0.2):
    """Generate white noise."""
    num_samples = int(SAMPLE_RATE * duration)
    samples = [amplitude * (random.random() * 2 - 1) for _ in range(num_samples)]
    return samples


def apply_envelope(samples, attack=0.01, decay=0.05, sustain=0.7, release=0.1):
    """Apply ADSR envelope to samples."""
    num_samples = len(samples)
    attack_samples = int(SAMPLE_RATE * attack)
    decay_samples = int(SAMPLE_RATE * decay)
    release_samples = int(SAMPLE_RATE * release)
    
    for i in range(num_samples):
        envelope = 1.0
        if i < attack_samples:
            envelope = i / attack_samples
        elif i < attack_samples + decay_samples:
            decay_progress = (i - attack_samples) / decay_samples
            envelope = 1.0 - (1.0 - sustain) * decay_progress
        elif i > num_samples - release_samples:
            release_progress = (num_samples - i) / release_samples
            envelope = sustain * release_progress
        else:
            envelope = sustain
        samples[i] *= envelope
    return samples


def apply_fade_out(samples, fade_duration=0.05):
    """Apply fade out to samples."""
    fade_samples = int(SAMPLE_RATE * fade_duration)
    for i in range(min(fade_samples, len(samples))):
        idx = len(samples) - 1 - i
        samples[idx] *= (i / fade_samples)
    return samples


def mix_samples(*sample_lists):
    """Mix multiple sample lists together."""
    max_len = max(len(s) for s in sample_lists)
    mixed = [0.0] * max_len
    for samples in sample_lists:
        for i, value in enumerate(samples):
            mixed[i] += value
    return mixed


def normalize_samples(samples, target_amplitude=0.8):
    """Normalize samples to target amplitude."""
    max_amplitude = max(abs(s) for s in samples)
    if max_amplitude > 0:
        scale = target_amplitude / max_amplitude
        samples = [s * scale for s in samples]
    return samples


def write_wav(filename, samples):
    """Write samples to WAV file."""
    samples = normalize_samples(samples)
    with wave.open(filename, 'w') as wav_file:
        wav_file.setnchannels(1)
        wav_file.setsampwidth(2)
        wav_file.setframerate(SAMPLE_RATE)
        for sample in samples:
            value = int(sample * 32767)
            value = max(-32768, min(32767, value))
            wav_file.writeframes(struct.pack('<h', value))
    file_size = os.path.getsize(filename)
    print(f"Generated {filename} ({file_size} bytes)")


def generate_fire_sound():
    """Generate weapon fire sound."""
    base = generate_square_wave(120, 0.08, 0.4)
    noise = generate_noise(0.08, 0.15)
    high = generate_sine_wave(800, 0.03, 0.2)
    
    samples = mix_samples(base, noise, high)
    samples = apply_envelope(samples, attack=0.001, decay=0.02, sustain=0.4, release=0.05)
    return samples


def generate_hit_sound():
    """Generate hit confirmation sound."""
    impact = generate_noise(0.02, 0.3)
    ping = generate_sine_wave(1200, 0.06, 0.25)
    
    samples = mix_samples(impact, ping)
    samples = apply_envelope(samples, attack=0.001, decay=0.01, sustain=0.5, release=0.03)
    return samples


def generate_frag_sound():
    """Generate frag/elimination sound."""
    low_thump = generate_sine_wave(80, 0.15, 0.5)
    mid_crunch = generate_noise(0.10, 0.2)
    high_sparkle = generate_sine_wave(2400, 0.20, 0.15)
    
    samples = mix_samples(low_thump, mid_crunch, high_sparkle)
    samples = apply_envelope(samples, attack=0.002, decay=0.05, sustain=0.6, release=0.1)
    return samples


def generate_round_start_sound():
    """Generate round start sound."""
    beep1 = generate_sine_wave(800, 0.08, 0.4)
    beep2 = generate_sine_wave(1000, 0.08, 0.4)
    
    silence = [0.0] * int(SAMPLE_RATE * 0.02)
    samples = beep1 + silence + beep2
    samples = apply_envelope(samples, attack=0.01, decay=0.02, sustain=0.8, release=0.05)
    return samples


def generate_round_end_sound():
    """Generate round end sound."""
    desc1 = []
    desc2 = []
    desc3 = []
    
    for freq_start, freq_end in [(600, 400), (500, 300), (400, 250)]:
        duration = 0.12
        num_samples = int(SAMPLE_RATE * duration)
        chunk = []
        for i in range(num_samples):
            t = i / SAMPLE_RATE
            progress = i / num_samples
            freq = freq_start + (freq_end - freq_start) * progress
            value = 0.3 * math.sin(2 * math.pi * freq * t)
            chunk.append(value)
        if freq_start == 600:
            desc1 = chunk
        elif freq_start == 500:
            desc2 = chunk
        else:
            desc3 = chunk
    
    samples = desc1 + desc2 + desc3
    samples = apply_fade_out(samples, 0.08)
    return samples


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    output_dir = os.path.join(script_dir, OUTPUT_DIR)
    os.makedirs(output_dir, exist_ok=True)
    
    print("Generating procedural audio for fragr...")
    
    sounds = {
        "fire.wav": generate_fire_sound,
        "hit.wav": generate_hit_sound,
        "frag.wav": generate_frag_sound,
        "round_start.wav": generate_round_start_sound,
        "round_end.wav": generate_round_end_sound,
    }
    
    for filename, generator in sounds.items():
        filepath = os.path.join(output_dir, filename)
        samples = generator()
        write_wav(filepath, samples)
    
    print(f"\nAll audio files generated in {output_dir}")
    total_size = sum(os.path.getsize(os.path.join(output_dir, f)) for f in sounds.keys())
    print(f"Total size: {total_size} bytes ({total_size / 1024:.1f} KB)")


if __name__ == "__main__":
    main()
