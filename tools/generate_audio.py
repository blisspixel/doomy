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
    """Generate weapon fire sound - snappy arcade punch."""
    kick = generate_sine_wave(80, 0.04, 0.6)
    snap = generate_noise(0.06, 0.4)
    high_crack = generate_square_wave(600, 0.04, 0.3)
    
    samples = mix_samples(kick, snap, high_crack)
    samples = apply_envelope(samples, attack=0.0005, decay=0.015, sustain=0.3, release=0.03)
    return samples


def generate_hit_sound():
    """Generate hit confirmation sound - satisfying arcade feedback."""
    thwack = generate_noise(0.015, 0.5)
    ding = generate_sine_wave(1800, 0.08, 0.4)
    sub_thump = generate_sine_wave(120, 0.03, 0.3)
    
    samples = mix_samples(thwack, ding, sub_thump)
    samples = apply_envelope(samples, attack=0.0005, decay=0.012, sustain=0.6, release=0.025)
    return samples


def generate_frag_sound():
    """Generate frag/elimination sound - SELL THE MOMENT, arcade glory."""
    massive_bass = generate_sine_wave(40, 0.25, 0.7)
    explosion_noise = generate_noise(0.12, 0.5)
    
    rising_sweep = []
    for i in range(int(SAMPLE_RATE * 0.15)):
        t = i / SAMPLE_RATE
        progress = i / (SAMPLE_RATE * 0.15)
        freq = 800 + progress * 1600
        value = 0.4 * math.sin(2 * math.pi * freq * t) * (1.0 - progress * 0.5)
        rising_sweep.append(value)
    
    sparkle_cascade = []
    for i in range(int(SAMPLE_RATE * 0.30)):
        t = i / SAMPLE_RATE
        progress = i / (SAMPLE_RATE * 0.30)
        freq1 = 2400 * (1.0 - progress * 0.3)
        freq2 = 3200 * (1.0 - progress * 0.4)
        decay_env = (1.0 - progress) ** 1.5
        value = decay_env * (0.25 * math.sin(2 * math.pi * freq1 * t) + 0.2 * math.sin(2 * math.pi * freq2 * t))
        sparkle_cascade.append(value)
    
    samples = mix_samples(massive_bass, explosion_noise, rising_sweep, sparkle_cascade)
    samples = apply_envelope(samples, attack=0.001, decay=0.08, sustain=0.7, release=0.15)
    return samples


def generate_round_start_sound():
    """Generate round start sound - arcade excitement, FIGHT!"""
    charge_up = []
    for i in range(int(SAMPLE_RATE * 0.10)):
        t = i / SAMPLE_RATE
        progress = i / (SAMPLE_RATE * 0.10)
        freq = 400 + progress * 400
        amp_env = progress ** 0.5
        value = 0.5 * amp_env * math.sin(2 * math.pi * freq * t)
        charge_up.append(value)
    
    impact_beep = generate_square_wave(1200, 0.10, 0.6)
    punch_bass = generate_sine_wave(100, 0.08, 0.5)
    
    gap = [0.0] * int(SAMPLE_RATE * 0.02)
    
    samples = charge_up + gap + mix_samples(impact_beep, punch_bass)
    samples = apply_envelope(samples, attack=0.005, decay=0.03, sustain=0.85, release=0.08)
    return samples


def generate_round_end_sound():
    """Generate round end sound - victorious fanfare or dramatic close."""
    victory_chord = []
    duration = 0.35
    num_samples = int(SAMPLE_RATE * duration)
    
    for i in range(num_samples):
        t = i / SAMPLE_RATE
        progress = i / num_samples
        
        root = 0.35 * math.sin(2 * math.pi * 440 * t)
        third = 0.28 * math.sin(2 * math.pi * 554 * t)
        fifth = 0.28 * math.sin(2 * math.pi * 659 * t)
        octave = 0.2 * math.sin(2 * math.pi * 880 * t)
        
        amp_env = (1.0 - progress) ** 0.6
        value = amp_env * (root + third + fifth + octave)
        victory_chord.append(value)
    
    bass_thump = generate_sine_wave(80, 0.15, 0.5)
    
    samples = mix_samples(victory_chord, bass_thump)
    samples = apply_fade_out(samples, 0.12)
    return samples



def generate_fire_flechette():
    """Mid chatter: snappy needle burst."""
    kick = generate_sine_wave(110, 0.03, 0.45)
    snap = generate_noise(0.045, 0.35)
    crack = generate_square_wave(900, 0.03, 0.28)
    samples = mix_samples(kick, snap, crack)
    return apply_envelope(samples, attack=0.0004, decay=0.012, sustain=0.28, release=0.02)


def generate_fire_rail():
    """Long precision: heavy charge crack + cold ring."""
    charge = generate_sine_wave(60, 0.05, 0.55)
    body = generate_sine_wave(220, 0.08, 0.35)
    crack = generate_square_wave(1400, 0.035, 0.22)
    ring = generate_sine_wave(2400, 0.10, 0.18)
    samples = mix_samples(charge, body, crack, ring)
    return apply_envelope(samples, attack=0.002, decay=0.03, sustain=0.45, release=0.06)


def generate_fire_scatter():
    """Close shred: chunky noise blast."""
    boom = generate_sine_wave(70, 0.05, 0.65)
    blast = generate_noise(0.09, 0.55)
    grit = generate_square_wave(280, 0.04, 0.25)
    samples = mix_samples(boom, blast, grit)
    return apply_envelope(samples, attack=0.0003, decay=0.02, sustain=0.35, release=0.04)


def generate_hit_flechette():
    """Needle thwack."""
    thwack = generate_noise(0.012, 0.45)
    ding = generate_sine_wave(1600, 0.06, 0.35)
    samples = mix_samples(thwack, ding)
    return apply_envelope(samples, attack=0.0004, decay=0.01, sustain=0.5, release=0.02)


def generate_hit_rail():
    """Heavy confirm: deep thump + cold ding."""
    thump = generate_sine_wave(90, 0.06, 0.5)
    ding = generate_sine_wave(2100, 0.10, 0.32)
    crack = generate_noise(0.02, 0.3)
    samples = mix_samples(thump, ding, crack)
    return apply_envelope(samples, attack=0.0005, decay=0.02, sustain=0.55, release=0.04)


def generate_hit_scatter():
    """Chunky flesh/metal splat."""
    splat = generate_noise(0.04, 0.55)
    thump = generate_sine_wave(100, 0.04, 0.4)
    grit = generate_square_wave(400, 0.03, 0.2)
    samples = mix_samples(splat, thump, grit)
    return apply_envelope(samples, attack=0.0003, decay=0.015, sustain=0.4, release=0.03)


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
        "fire_flechette.wav": generate_fire_flechette,
        "fire_rail.wav": generate_fire_rail,
        "fire_scatter.wav": generate_fire_scatter,
        "hit_flechette.wav": generate_hit_flechette,
        "hit_rail.wav": generate_hit_rail,
        "hit_scatter.wav": generate_hit_scatter,
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
