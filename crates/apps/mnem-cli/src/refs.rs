use anyhow::Result;

pub fn handle_refs(symbol: Option<String>, _since: Option<String>) -> Result<()> {
    if let Some(ref s) = symbol {
        println!("References to '{}':", s);
        println!("(Reference finding - coming soon)");
        return Ok(());
    }

    println!("Usage: mnem refs <symbol>");

    Ok(())
}
