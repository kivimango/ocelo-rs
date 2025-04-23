fn main() {
    let cpu_info = core::get_cpu_info();

    println!("🧠 CPU Info:");
    println!("  Name: {}", cpu_info.name);
    println!("  Cores: {}", cpu_info.core_count);
    println!("  Avg Frequency: {} MHz", cpu_info.frequency);

    if let Some(temp) = cpu_info.temperature {
        println!("  Temperature: {:.1}°C", temp);
    } else {
        println!("  Temperature: N/A");
    }
}
