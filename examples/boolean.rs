fn main() {
    let enabled = obfany::obfbool!(true);
    let disabled = obfany::obfbool!(false);

    println!("enabled: {}", enabled);
    println!("disabled: {}", disabled);
}
