use std::fs::File;
use std::io::Write;
use crate::{EvaluatedHiscores, EvaluatedCategory, EvaluatedEntry};


pub fn write_index(data: Vec<(String, u32, u32, u32, u32, String)>) -> std::io::Result<()> {

    // Define the HTML template
    let mut html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>FE Nixes Clan Stats</title>
    <link rel="stylesheet" href="style.css">
    <script>
            function goToDetailsPage(username) {
                // Redirect to details.html with the username as a query parameter
                window.location.href = 'out/details/' + username + '.html';
            }
        </script>
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
            <tr onclick="goToDetailsPage('{}')">
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>
            "#,
            name, name, total_kills, boss_kills, deaths, points, best_drop
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

pub fn generate_hiscores_details_page(username: &str, hiscores: &EvaluatedHiscores) -> String {
    let mut html = String::from(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
        <link rel="stylesheet" href="../../style.css">
        </head>
        <body>
        <h1>Details for {}</h1>
        <h2>Total Points: {}</h2>
    "#,
        username, hiscores.points
    ));

    for category in &hiscores.categories {
        html.push_str(&format!(
            r#"
            <div class="category">
            <h3>{}</h3>
            <table>
            <tr>
                <th>Entry</th>
                <th>Score</th>
                <th>Points</th>
            </tr>
            "#,
            category.name
        ));

        for entry in &category.evaluated_entries {
            html.push_str(&format!(
                r#"
                <tr>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
                "#,
                entry.name, entry.score, entry.points
            ));
        }

        html.push_str(
            r#"
            </table>
            </div>
            "#,
        );
    }

    html.push_str(
        r#"
        </body>
        </html>
        "#,
    );

    html
}

pub fn save_hiscores_details_page(username: &str, hiscores: &EvaluatedHiscores) -> std::io::Result<()> {
    println!("{:?}", username);
    let html = generate_hiscores_details_page(username, hiscores);
    let filename = format!("out/details/{}.html", username);
    let mut file = File::create(filename)?;
    file.write_all(html.as_bytes())?;
    Ok(())
}