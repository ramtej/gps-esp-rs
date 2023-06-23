#[cfg(feature = "target-esp32-none-elf")]
use esp_idf_sys as _;
use num_complex::Complex32;
use std::io;

use crate::cacode::{CACode, SVs};
use rustfft::{num_complex::Complex, FftPlanner};

pub mod cacode;

const LO_SIN: [i32; 4] = [1, 1, 0, 0];
const LO_COS: [i32; 4] = [1, 0, 0, 1];

fn read_sdr_file_1bit_from_static_array(start: u64, len: usize) -> io::Result<Vec<u8>> {
    const IQ: &[u8] = include_bytes!("../data/gps.samples.1bit.I.fs2800KSPS.if620K.bin.rtl");

    let mut sample_data = vec![0; len * 8];
    let mut buffer = [0; 1];
    let mut idx = 0;

    while idx < len {
        buffer[0] = *IQ.get((start as usize) + idx).ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "unexpected end of data")
        })?;
        let mut ch = buffer[0];
        //println!("ch: {}", ch);
        for j in 0..8 {
            sample_data[(idx * 8) + j] = ch & 1; // Adjusted for bit-level operations
            ch >>= 1;
        }
        idx += 1; // Adjusted to increment by 1 instead of 8
    }

    Ok(sample_data)
}

fn main() {
    #[cfg(feature = "target-esp32-none-elf")]
    {
        esp_idf_sys::link_patches();
        esp_idf_svc::log::EspLogger::initialize_default();
    }

    let fc = 620000;
    // let fs = 2800000;
    let fs = 2800000;
    let ms = 5; // Length of data to process (milliseconds)
    let len_per_sec = ms * fs / 1000;

    let ca_freq: f32 = 1023000.0;
    let mut ca_phase: f32 = 0.0;
    let ca_rate: f32 = ca_freq / fs as f32;

    let samples_iq = read_sdr_file_1bit_from_static_array(0, len_per_sec).unwrap();

    let mut code: Vec<Complex32> = vec![Complex32 { re: 0.0, im: 0.0 }; len_per_sec];
    let mut data: Vec<Complex32> = vec![Complex32 { re: 0.0, im: 0.0 }; len_per_sec];
    let mut prod: Vec<Complex32> = vec![Complex32 { re: 0.0, im: 0.0 }; len_per_sec];

    println!("GPS L1 C/A Code Search on [{}].", std::env::consts::ARCH);
    println!("============================");
    println!(
        "Processing {} ms of GPS L1 1bit signal @({} SPS)",
        ms,
        len_per_sec as f32 / (ms as f32 * 1000. / 1_000_000.)
    );

    println!(" ");

    println!(
        "{:<10} {:<15} {:<18} {:<10}",
        "SV_Id", "Doppler(Hz)", "Phase(Offset)", "Max_SNR"
    );

    println!(" ");

    // FFTs
    let fft = FftPlanner::<f32>::new().plan_fft_forward(len_per_sec);
    let ifft = FftPlanner::<f32>::new().plan_fft_inverse(len_per_sec);

    let svs = SVs::new();

    for (sv_id, sv_params) in svs.svs {
        let mut ca = CACode::new(sv_params.t1 as usize, sv_params.t2 as usize);

        (0..len_per_sec).for_each(|i| {
            code[i] = if ca.chip() {
                Complex::new(-1.0, 0.0)
            } else {
                Complex::new(1.0, 0.0)
            };

            ca_phase += ca_rate;
            // clock only when phase is changing related to sample rate
            if ca_phase >= 1.0 {
                ca_phase -= 1.0;
                ca.clock();
            }
        });

        /******************************************
         * Now run the FFT on the C/A code stream  *
         ******************************************/

        fft.process(&mut code);
        let code_out = code.clone();

        /******************************************
         * Now run the FFT on the C/A code stream  *
         ******************************************/

        let lo_freq = fc as f32;
        let mut lo_phase: f32 = 0.0;
        let lo_rate = lo_freq / (fs as f32) * 4.0;

        for i in 0..len_per_sec {
            data[i] = if (samples_iq[i] ^ LO_SIN[lo_phase.floor() as usize] as u8) != 0 {
                Complex::new(-1.0, 0.0)
            } else {
                Complex::new(1.0, 0.0)
            };

            data[i].im = if (samples_iq[i] ^ LO_COS[lo_phase.floor() as usize] as u8) != 0 {
                -1.0
            } else {
                1.0
            };

            lo_phase += lo_rate;
            if lo_phase >= 4.0 {
                lo_phase -= 4.0;
            }
        }

        fft.process(&mut data);
        let data_out = data.clone();

        let mut max_snr = 0.0;
        let mut max_snr_dop = 0;
        let mut max_snr_i = 0.0;

        let start = (-5000.0 * len_per_sec as f32 / fs as f32) as i32;
        let end = (5000.0 * len_per_sec as f32 / fs as f32) as i32;

        for dop in start..=end {
            let dop = dop;
            let mut max_pwr: f32 = 0.0;
            let mut tot_pwr: f32 = 0.0;
            let mut max_pwr_i: f32 = 0.0;

            for i in 0..len_per_sec {
                let j = (i as i32 + len_per_sec as i32 - dop) % len_per_sec as i32;
                let j = j as usize;
                prod[i] = Complex::new(
                    data_out[i].re * code_out[j].re + data_out[i].im * code_out[j].im,
                    data_out[i].re * code_out[j].im - data_out[i].im * code_out[j].re,
                );
            }

            ifft.process(&mut prod);
            let prod_out = prod.clone();

            let mut ii = 0;
            (0..fs / 1000).for_each(|i| {
                ii = i;
                //let pwr = prod_out[i].norm_sqr();
                let pwr = prod_out[i].re * prod_out[i].re + prod_out[i].im * prod_out[i].im;

                if pwr > max_pwr {
                    max_pwr = pwr;
                    max_pwr_i = i as f32;
                }
                tot_pwr += pwr;
            });

            let ave_pwr = tot_pwr / ii as f32;
            let snr = max_pwr / ave_pwr;
            if snr > max_snr {
                max_snr = snr;
                max_snr_dop = dop;
                max_snr_i = max_pwr_i;
            };
        }

        // sv_id

        let doppler = max_snr_dop as f32 * fs as f32 / len_per_sec as f32;
        let phase = (max_snr_i * 1023.0) / (fs as f32 / 1000.0);

        println!(
            "{:<10} {:<15} {:<18} {:<10} {:<}",
            sv_id,
            doppler,
            phase,
            max_snr,
            "*".repeat((max_snr as i32 / 10) as usize)
        );
    }
}
