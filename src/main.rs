mod models;
mod formatters;
mod auth;
mod api;

use reqwest::Client;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read credentials from file
    let credentials = match fs::read_to_string("credentials.txt") {
        Ok(c) => c,
        Err(_) => {
            eprintln!("credentials.txt not found. Please create it with email on first line and password on second.");
            return Ok(());
        }
    };
    let lines: Vec<&str> = credentials.lines().collect();
    if lines.len() < 2 {
        eprintln!("credentials.txt must have at least 2 lines: email and password");
        return Ok(());
    }
    let email = lines[0].to_string();
    let password = lines[1].to_string();

    // Create GraphQL request
    let request = models::GraphQLRequest {
        query: "mutation login($u: String!, $p: String!) { login(u: $u, p: $p) }".to_string(),
        variables: models::LoginVariables { u: email, p: password },
    };

    // Send request
    let client = Client::new();
    let response = api::login_request(&client, &request).await?;

    if response.data.is_some() {
        let data = response.data.as_ref().unwrap();
        println!("Login successful. Token: {}", data.login);
        // Decode token to get uid
        let claims = auth::decode_token(&data.login)?;
        let uid = claims.id;
        println!("User ID: {}", uid);

        // Now retrieve latest workout
        let workout_request = models::WorkoutRequest {
            query: r#"
query JDay($uid: ID!, $ymd: YMD) {
  jday(uid: $uid, ymd: $ymd) {
    log
    bw
    eblocks {
      eid
      sets {
        w
        r
        s
        lb
        rpe
        pr
        est1rm
        eff
        int
        type
        t
        d
        dunit
        speed
        force
      }
    }
    exercises {
      exercise {
        id
        name
        type
      }
    }
  }
}
            "#.to_string(),
            variables: models::WorkoutVariables { uid, ymd: Some("2025-10-31".to_string()) },
        };

        let workout_body = api::workout_request(&client, &data.login, &workout_request).await?;
        if let Some(errors) = workout_body.errors {
            for error in errors {
                eprintln!("GraphQL Error: {}", error.message);
            }
        } else if let Some(data) = workout_body.data {
            if let Some(jday) = data.jday {
                println!("Full formatted workout:\n{}", formatters::format_full_workout(&jday, "2025-10-31"));
            } else {
                println!("No workout found for the date.");
            }
        } else {
            println!("Unexpected response.");
        }
    } else if let Some(errors) = response.errors {
        for error in errors {
            eprintln!("Error: {}", error.message);
        }
    } else {
        eprintln!("Unexpected response");
    }

    Ok(())
}