use std::path::Path;

pub fn make_path_relative(path: &str) -> String {
    let current_dir = std::env::current_dir().unwrap_or_default();
    let path_buf = Path::new(path);

    if let Ok(rel) = path_buf.strip_prefix(&current_dir) {
        rel.to_string_lossy().to_string()
    } else {
        // Fallback: show only the last 3 components if too long
        let components: Vec<_> = path_buf.components().collect();
        if components.len() > 3 {
            let last_3: Vec<_> = components
                .iter()
                .rev()
                .take(3)
                .rev()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect();
            format!(".../{}", last_3.join("/"))
        } else {
            path.to_string()
        }
    }
}

pub fn paginate_per_branch<T, F>(items: Vec<T>, limit: usize, get_branch: F) -> Vec<T>
where
    F: Fn(&T) -> Option<String>,
{
    use std::collections::HashMap;
    let mut counts = HashMap::new();
    let mut result = Vec::new();

    for item in items {
        let branch = get_branch(&item).unwrap_or_else(|| "main".to_string());
        let count = counts.entry(branch).or_insert(0);
        if *count < limit {
            result.push(item);
            *count += 1;
        }
    }
    result
}

pub struct CommonArgs {
    pub limit: usize,
    pub branch: Option<String>,
}

pub fn parse_common_args(args: &[String]) -> CommonArgs {
    let limit = args
        .iter()
        .position(|a| a == "--limit" || a == "-l")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10);

    let branch = args
        .iter()
        .position(|a| a == "--branch" || a == "-b")
        .and_then(|i| args.get(i + 1))
        .cloned();

    CommonArgs { limit, branch }
}
