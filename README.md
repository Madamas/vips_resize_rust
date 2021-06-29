# Image resize via native Rust image processing capabilities

## Usage
To start project just run `cargo run` - the project will be hosted on `0.0.0.0:3030/`

Service accepts GET requests on root route in next format:
```curl
http://0.0.0.0:3000/thumbnail?url=url-to-image&width=180
```

* Converts png, jpeg and gif images