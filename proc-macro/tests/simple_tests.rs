use unistructgen_macro::struct_from_external_api;

struct_from_external_api! {
    struct_name = "User",
    url_api = "https://jsonplaceholder.typicode.com/users/1",

}

// Example 2: Fetch a single post
struct_from_external_api! {
    struct_name = "Post",
    url_api = "https://jsonplaceholder.typicode.com/posts/1"
}

// Example 3: Fetch todos with depth limit (avoid nested objects)
struct_from_external_api! {
    struct_name = "Todo",
    url_api = "https://jsonplaceholder.typicode.com/todos/1",
    max_depth = 2
}

// Example 4: Fetch comments with custom timeout
struct_from_external_api! {
    struct_name = "Comment",
    url_api = "https://jsonplaceholder.typicode.com/comments/1",
    request_timeout = 5000
}

#[test]
fn main_tests() {
    println!("=== API-Generated Structs Demo ===\n");

    // Example 1: Single User
    println!("Example 1: Single User");
    let user = User {
        id: 1,
        name: "Leanne Graham".to_string(),
        username: "Bret".to_string(),
        email: "Sincere@april.biz".to_string(),
        address: Address {
            street: "Kulas Light".to_string(),
            suite: "Apt. 556".to_string(),
            city: "Gwenborough".to_string(),
            zipcode: "92998-3874".to_string(),
            geo: Geo {
                lat: "-37.3159".to_string(),
                lng: "81.1496".to_string(),
            },
        },
        phone: "1-770-736-8031 x56442".to_string(),
        website: "hildegard.org".to_string(),
        company: Company {
            name: "Romaguera-Crona".to_string(),
            catch_phrase: "Multi-layered client-server neural-net".to_string(),
            bs: "harness real-time e-markets".to_string(),
        },
    };
    println!("User: {:?}", user.name);
    println!("Email: {:?}", user.email);
    println!("City: {:?}\n", user.address.city);

    // Example 2: Single Post
    println!("Example 2: Single Post");
    let post = Post {
        user_id: 1,
        id: 1,
        title: "sunt aut facere repellat provident".to_string(),
        body: "quia et suscipit...".to_string(),
    };
    println!("Post ID: {}", post.id);
    println!("Title: {}\n", post.title);

    // Example 3: Todo
    println!("Example 3: Todo");
    let todo = Todo {
        user_id: 1,
        id: 1,
        title: "delectus aut autem".to_string(),
        completed: false,
    };
    println!("Todo: {}", todo.title);
    println!("Completed: {}\n", todo.completed);

    // Example 4: Comment
    println!("Example 4: Comment");
    let comment = Comment {
        post_id: 1,
        id: 1,
        name: "id labore ex et quam laborum".to_string(),
        email: "Eliseo@gardner.biz".to_string(),
        body: "laudantium enim quasi est quidem magnam...".to_string(),
    };
    println!("Comment from: {}", comment.email);
    println!("Body: {}\n", comment.body);

    println!("=== All examples compiled successfully! ===");
    println!("Structs were generated at compile-time from real API calls.");
}