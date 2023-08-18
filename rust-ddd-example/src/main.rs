use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

mod query {
    pub struct Query {}
    pub struct QueryBus {}
    pub struct Response {}
}

mod command {
    use enum_dispatch::enum_dispatch;
    use std::collections::HashMap;

    pub struct Mediator {
        handlers: HashMap<String, Box<dyn CommandHandler>>,
    }
    pub struct FirstCommand {
        first_param: String,
    }

    //List operations
    #[enum_dispatch]
    pub enum Command {
        FirstCommand,
    }

    #[enum_dispatch(Command)]
    trait FeatureCommand {
        fn name(&self) -> String;
    }

    impl FeatureCommand for FirstCommand {
        fn name(&self) -> String {
            "FirstCommand".to_string()
        }
    }

    pub trait CommandHandler {
        fn run(&self, command: Command);
        fn name(&self) -> String;
    }

    // Entrypoint for the command handler
    impl Mediator {
        pub fn register(&mut self, handler: Box<dyn CommandHandler>) {
            let name_key: String = (*handler.name()).to_string();
            *self.handlers.get_mut(&name_key).unwrap() = handler;
        }
        pub fn dispatch(&self, command: Command) {
            if let Some(entry) = self.handlers.get(&command.name()) {
                entry.run(command);
            }
        }
    }

    // Application layer
    struct FirstCommandHandler {}
    impl CommandHandler for FirstCommandHandler {
        fn run(&self, command: Command) {
            let Command::FirstCommand(val) = command;
            println!("{:?}", val.first_param);
        }
        fn name(&self) -> String {
            "FirstCommand".to_string()
        }
    }

    struct SaveVideo {}
    impl SaveVideo {
        pub fn run() {}
    }
}

pub struct ApiController {
    pub commandBus: command::Mediator,
    pub queryBus: query::QueryBus,
}

impl ApiController {
    pub fn ask() -> query::Response {
        query::Response {}
    }
    pub fn dispatch(&self, command: command::Command) {
        self.commandBus.dispatch(command);
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
