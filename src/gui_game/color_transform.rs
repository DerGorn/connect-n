pub fn hsva_to_rgba(hsva: [u8; 4]) -> [u8; 4] {
    let [h, s, l, a] = hsva;
    let h_f = h as f64 / 255.0;
    let s_f = s as f64 / 100.0;
    let v_f = l as f64 / 100.0;

    // println!("{}, {}, {}, {}");

    let c = v_f * s_f;
    let h_dash = h_f * 6.0;
    let x = c * (1.0 - (h_dash % 2.0 - 1.0).abs());

    let m = v_f - c;
    let c = ((c + m) * 255.0) as u8;
    let x = ((x + m) * 255.0) as u8;
    let m = (m * 255.0) as u8;

    match h_dash {
        f if f < 1.0 => [c, x, m, a],
        f if f < 2.0 => [x, c, m, a],
        f if f < 3.0 => [m, c, x, a],
        f if f < 4.0 => [m, x, c, a],
        f if f < 5.0 => [x, m, c, a],
        f if f < 6.0 => [c, m, x, a],
        _ => panic!("Fucking Fuck fuck"),
    }
}
