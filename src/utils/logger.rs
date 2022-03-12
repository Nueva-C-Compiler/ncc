
fn log(msg: String) { 
    let text = textwrap::wrap(text, 80);

    let corner = boxy::Char::upper_left(boxy::Weight::Doubled);
    let side = boxy::Char::horizontal(boxy::Weight::Doubled);


    println!("{}{}{}", corner, (0..78).map(|_| side).collect::<String>(), corner.rotate_cw(1));
    for i in text {
	// println!("{} {} {}
    }
    
    let bx = format!(
	"{}{}{}\n{}{}{}",
	corner, side, corner.rotate_cw(1),
	corner.rotate_cw(3), side, corner.rotate_cw(2),
    );
    
    println!(bx);
}

