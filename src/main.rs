use evtx::EvtxParser;
use serde_json::Value;
use std::env;
use std::fs;
use std::collections::HashSet;

fn main() {
    // let input_file = env::args().nth(1).expect("Error reading args");
    // let mut events = read_file(&input_file);
    
    let mut events = read_directory("./test-logs");

    let mut events = filter_common_words(events);
    let mut events = parse_events(events);
    events.sort_by(|a,b| b.len().partial_cmp(&a.len()).unwrap());

    for e in events {
        println!("{}", e);
    }
}

fn read_file(file_name: &str) -> Vec<String> {
    let events = read_events(file_name);
    return split_events(&events);
}

fn read_directory(path: &str) -> Vec<String> {

    let paths = fs::read_dir(path).unwrap();
    let mut events = Vec::new();

    for path in paths {
        match path {
            Ok(p) => events.append(&mut read_file(&p.path().to_string_lossy())),
            Err(_) => (),
        }
    }

    return events
}

fn parse_events(mut events: Vec<String>) -> Vec<String> {
    let mut filtered_events = Vec::<String>::with_capacity(events.len());

    //find stuff that end in '=' or '/'
    //find stuff divisible by 4
    let mut i = 0;
    while i < events.len() {
        let last_char = match events[i].chars().last() {
            Some(val) => val,
            None => continue,
        };

        if last_char == '=' || last_char == '/' {
            filtered_events.push(events.swap_remove(i));
        }
        else if events[i].len() % 4 == 0 {
            filtered_events.push(events.swap_remove(i));
        }
        else {
            i += 1;
        }
    }

    return filtered_events;
}

fn filter_common_words(mut events: Vec<String>) -> Vec<String> {
    let common_words = vec!["Local", "Data", "Windows", "Install", "Update", "User", "Common", "Temp", "Default", "Enable", "data", "Document", "Direct"];
    let mut filtered_events = Vec::new();

    let mut i = 0;
    'event_loop: while i < events.len() {

        //we don't want the event if it has one of the common_words in it, so we don't add it to
        //the vec if it is in the event
        // for word in &common_words {
        //     if events[i].contains(word) {
        //         i += 1;
        //         continue 'event_loop;
        //     }
        // }
        if common_words.iter().any(|x| events[i].contains(x)) {
            i += 1;
            continue;
        }

        filtered_events.push(events.swap_remove(i));
    }

    return filtered_events;
}

fn split_events(events: &[Value]) -> Vec<String> {
    let mut found_splits = HashSet::with_capacity(events.len());

    for event in events {
        let mut event_data = String::new();

        match &event["Event"]["EventData"]["CommandLine"] {
            Value::Null => (),
            val => {
                event_data = format!("{} {}", event_data, val);
            },
        };

        match &event["Event"]["EventData"]["ScriptBlockText"] {
            Value::Null => (),
            val => {
                event_data = format!("{} {}", event_data, val);
            },
        };

        if event_data == "" {
            continue;
        }

        let event_split: Vec<_> = event_data.split(|c| c == ' ' || c == '(' || c == ')' || c == '#' || c == '.' || c == ',' || c == '\'' || c == '{' || c == '}' || c == '\"' || c == '-' || c == '*' || c == '_' || c == '<' || c == '>')
            // .filter(|x| x != &"")
            .filter(|x| x.len() > 50)
            .map(|x| x.to_string())
            .collect();

        for e in event_split {
            found_splits.insert(e);
        }
    }

    return found_splits.into_iter().collect();
}

fn read_events(path: &str) -> Vec<Value> {
    let mut parser = match EvtxParser::from_path(path) {
        Ok(val) => val,
        Err(_) => {
            dbg!(path);
            return Vec::new();
        },
    };

    let mut events = Vec::<Value>::new();

    for record in parser.records_json() {
        match record {
            Ok(r) => {
                events.push(serde_json::from_str(&r.data).unwrap());
            },
            Err(e) => eprintln!("{}", e),
        }
    }

    events
}


