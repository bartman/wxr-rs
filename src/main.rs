use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: LoginVariables,
}

#[derive(Serialize)]
struct LoginVariables {
    u: String,
    p: String,
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<LoginData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct LoginData {
    login: String,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
struct Claims {
    id: u32,
}

#[derive(Serialize)]
struct WorkoutRequest {
    query: String,
    variables: WorkoutVariables,
}

#[derive(Serialize)]
struct WorkoutVariables {
    uid: u32,
    ymd: Option<String>,
}

#[derive(Deserialize)]
struct WorkoutResponse {
    data: Option<WorkoutData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct WorkoutData {
    jday: Option<JDay>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JDay {
    log: String,
    bw: Option<f32>,
    eblocks: Vec<EBlock>,
    exercises: Vec<ExerciseWrapper>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct EBlock {
    eid: String,
    sets: Vec<Set>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Set {
    w: Option<f32>,
    r: Option<u32>,
    s: Option<u32>,
    lb: Option<f32>,
    rpe: Option<f32>,
    pr: Option<i32>,
    est1rm: Option<f32>,
    eff: Option<f32>,
    int: Option<f32>,
    #[serde(rename = "type")]
    set_type: Option<i32>,
    t: Option<f32>,
    d: Option<f32>,
    dunit: Option<String>,
    speed: Option<f32>,
    force: Option<f32>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ExerciseWrapper {
    exercise: Exercise,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Exercise {
    id: String,
    name: String,
    #[serde(rename = "type")]
    ex_type: String,
}

fn format_weight(w: f32, lb: bool) -> String {
    if lb {
        format!("{:.0}", w * 2.20462)
    } else {
        format!("{:.0}", w)
    }
}

fn format_set(set: &Set) -> String {
    let w = set.w.unwrap_or(0.0);
    let r = set.r.unwrap_or(0);
    let s = set.s.unwrap_or(1);
    let rpe = set.rpe.unwrap_or(0.0);
    let lb = set.lb.unwrap_or(0.0) == 1.0;
    let w_str = format_weight(w, lb);
    let mut line = w_str;
    if r > 0 {
        line += &format!(" x {}", r);
    }
    if s > 1 {
        line += &format!(" x {}", s);
    }
    if rpe > 0.0 {
        line += &format!(" @{}", rpe);
    }
    line
}

fn compress_sets(sets: &Vec<Set>) -> Vec<String> {
    let mut compressed = Vec::new();
    let mut i = 0;
    while i < sets.len() {
        let set = &sets[i];
        if set.set_type.unwrap_or(0) != 0 {
            compressed.push(format_set(set));
            i += 1;
            continue;
        }
        let w = set.w.unwrap_or(0.0);
        let r = set.r.unwrap_or(0);
        let _s = set.s.unwrap_or(1);
        let rpe = set.rpe.unwrap_or(0.0);
        let lb = set.lb.unwrap_or(0.0) == 1.0;
        // check for same weight consecutive
        let mut same_weight = vec![r];
        let mut j = i + 1;
        while j < sets.len() {
            let next = &sets[j];
            if next.set_type.unwrap_or(0) != 0 || next.w != set.w || next.rpe != set.rpe || next.lb != set.lb || next.s != set.s {
                break;
            }
            same_weight.push(next.r.unwrap_or(0));
            j += 1;
        }
        if same_weight.len() > 1 {
            let w_str = format_weight(w, lb);
            let r_str = same_weight.iter().map(|&r| r.to_string()).collect::<Vec<_>>().join(", ");
            let mut line = format!("{} x {}", w_str, r_str);
            if rpe > 0.0 {
                line += &format!(" @{}", rpe);
            }
            compressed.push(line);
            i = j;
        } else {
            // check for same rep
            let mut same_rep = vec![w];
            let mut j = i + 1;
            while j < sets.len() {
                let next = &sets[j];
                if next.set_type.unwrap_or(0) != 0 || next.r != set.r || next.rpe != set.rpe || next.lb != set.lb || next.s != set.s {
                    break;
                }
                same_rep.push(next.w.unwrap_or(0.0));
                j += 1;
            }
            if same_rep.len() > 1 {
                let w_str = same_rep.iter().map(|&w| format_weight(w, lb)).collect::<Vec<_>>().join(", ");
                let mut line = format!("{} x {}", w_str, r);
                if rpe > 0.0 {
                    line += &format!(" @{}", rpe);
                }
                compressed.push(line);
                i = j;
            } else {
                compressed.push(format_set(set));
                i += 1;
            }
        }
    }
    compressed
}

fn format_workout(jday: &JDay) -> String {
    let mut ex_map: HashMap<String, &Exercise> = HashMap::new();
    for ex_wrap in &jday.exercises {
        ex_map.insert(ex_wrap.exercise.id.clone(), &ex_wrap.exercise);
    }
    let mut lines = Vec::new();
    for eblock in &jday.eblocks {
        if let Some(ex) = ex_map.get(&eblock.eid) {
            lines.push("#".to_string() + &ex.name);
            lines.extend(compress_sets(&eblock.sets));
        }
    }
    lines.join("\n")
}

fn format_full_workout(jday: &JDay, ymd: &str) -> String {
    let mut lines = vec![ymd.to_string()];
    if let Some(bw) = jday.bw {
        lines.push(format!("@ {:.0} bw", bw * 2.20462));
    }
    let formatted_sets = format_workout(jday);
    let full_log = jday.log.replace("EBLOCK:137778", &("\n".to_string() + &formatted_sets + "\n"));
    lines.push(full_log);
    lines.join("\n")
}

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
    let request = GraphQLRequest {
        query: "mutation login($u: String!, $p: String!) { login(u: $u, p: $p) }".to_string(),
        variables: LoginVariables { u: email, p: password },
    };

    // Send request
    let client = Client::new();
    let response = client
        .post("https://weightxreps.net/api/graphql")
        .json(&request)
        .send()
        .await?;

    let status = response.status();
    let body: GraphQLResponse = response.json().await?;

    if status.is_success() {
        if let Some(data) = body.data {
            println!("Login successful. Token: {}", data.login);
            // Decode token to get uid
            let parts: Vec<&str> = data.login.split('.').collect();
            if parts.len() != 3 {
                eprintln!("Invalid token format");
                return Ok(());
            }
            let payload = parts[1];
            let decoded = general_purpose::URL_SAFE_NO_PAD.decode(payload).unwrap();
            let claims: Claims = serde_json::from_slice(&decoded).unwrap();
            let uid = claims.id;
            println!("User ID: {}", uid);

            // Now retrieve latest workout
            let workout_request = WorkoutRequest {
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
                variables: WorkoutVariables { uid, ymd: Some("2025-10-31".to_string()) },
            };

            let workout_response = client
                .post("https://weightxreps.net/api/graphql")
                .header("Authorization", format!("Bearer {}", data.login))
                .json(&workout_request)
                .send()
                .await?;

            let workout_body: WorkoutResponse = workout_response.json().await?;
            if let Some(errors) = workout_body.errors {
                for error in errors {
                    eprintln!("GraphQL Error: {}", error.message);
                }
            } else if let Some(data) = workout_body.data {
                if let Some(jday) = data.jday {
                    println!("Full formatted workout:\n{}", format_full_workout(&jday, "2025-10-31"));
                } else {
                    println!("No workout found for the date.");
                }
            } else {
                println!("Unexpected response.");
            }
        } else if let Some(errors) = body.errors {
            for error in errors {
                eprintln!("Error: {}", error.message);
            }
        } else {
            eprintln!("Unexpected response");
        }
    } else {
        eprintln!("HTTP error: {}", status);
    }

    Ok(())
}