use std::fs::File;
use std::io::Write;

pub fn write_file(data: Vec<(String, u32, u32, u32, u32, String)>) -> std::io::Result<()> {

    // Define the HTML template
    let mut html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FE Nixes Clan Stats</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <h1>FE Nixes Clan - Player Stats</h1>
    <table>
        <thead>
            <tr>
                <th>Player Name</th>
                <th>Total Points</th>
                <th>Points from Skilling</th>
                <th>Points from Clues and Activities</th>
                <th>Points from PVM</th>
                <th>Fe Nixes Rank</th>
            </tr>
        </thead>
        <tbody>
"#.to_string();

    // Append table rows dynamically
    for (name, total_kills, boss_kills, deaths, points, best_drop) in data {
        html_content.push_str(&format!(
            r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
            "#,
            name, total_kills, boss_kills, deaths, points, best_drop
        ));
    }

    // Close the table and body
    html_content.push_str(r#"
        </tbody>
    </table>
</body>
</html>
"#);

    // Write the content to index.html
    let mut file = File::create("index.html")?;
    file.write_all(html_content.as_bytes())?;

    println!("index.html file generated successfully!");
    Ok(())
}