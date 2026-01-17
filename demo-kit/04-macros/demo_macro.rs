use unistructgen_macro::struct_from_external_api;

// Defines a struct 'Todo' by fetching data from this URL at compile time!
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://jsonplaceholder.typicode.com/todos/1"
}

fn main() {
    let todo = Todo {
        user_id: 1,
        id: 101,
        title: "Learn Rust Macros".to_string(),
        completed: false,
    };

    println!("Task: {} (Completed: {})", todo.title, todo.completed);
}
