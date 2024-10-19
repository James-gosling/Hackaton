# Receiver contract

When compiling (inside the wasm folder or in the root path), two files will be created, "app.idl" which specifies the types, services, etc; and "app_client.rs (contains all the necessary code) which will be used to communicate with this contract (receiver client), both files will be inside the "wasm" directory.

To upload the contract, you have to go to [IDEA](https://idea.gear-tech.io/programs?node=wss%3A%2F%2Ftestnet.vara.network) and upload the .opt.wasm (in target/wasm32--https://idea.gear-tech.io/programs?node=wss%3A%2F%2Ftestnet.vara.network/release) and idl files that were generated

To be able to communicate with this contract, you need to copy the "app_client.rs" on your contract files and use it in your code.

## First steps to use a client

In your "app" directory, you can create a module called "clients" where you can store the client for this contract. The directory tree would look like this: 

<pre>
    app
    ├── Cargo.toml <- file where dependencies are specified
    └── src <- Here you will find the contract files and directories
        ├── clients <- Directory where all services are stored
        |   ├── mod.rs file that specifies the contract clients module
        |   └── app_client.rs <- Client code used to communicate with the contract
        ├── services <- Directory where all services are stored
        |   ├── mod.rs file that specifies the contract services module
        |   └── contract_service.rs <- Contraxt service example
        ├── states <- Directory where all contract states are stored
        └── lib.rs <- file where the contract "program" is created
</pre>

In the mod.rs that is in "clients" directory, you need to put the next lines (to import clients, enums, etc):

```rust
pub mod app_client;
```

And in your "lib.rs" file, yo have to put the next lines:

```rust
pub mod clients;

use clients::app_client::{
    Receiver as ReceiverClient,
    QuerySvc as ReceiverQueryClient
};
```

Then, you will use RefCell to store your data (contract state) in the program itself (more information of [RefCell](https://doc.rust-lang.org/stable/book/ch15-05-interior-mutability.html)), to reduce the number of tokens that the program will use in the future when it call to the contract (with the client), you can save the client in the state (because each service is stateless), you need to import RefCell, with the above it would be like this (in lib.rs file):

```rust
use sails_rs::{
    prelude::*,
    cell::RefCell,
    gstd::calls::GStdRemoting
};

pub mod services; // set services module
pub mod clients;

use clients::app_client::{
    Receiver as ReceiverClient,
    QuerySvc as ReceiverQueryClient
};


pub struct MyProgram {
    pub receiver_client: RefCell<ReceiverClient<GStdRemoting>>,
    pub receiver_query_client: RefCell<ReceiverQueryClient<GStdRemoting>>
}

#[program]
impl MyProgram {
    // program initializer with both clients
    pub fn new() -> Self {
        // Set first client 
        let receiver_client = RefCell::new(
            ReceiverClient::new(GStdRemoting)
        );

        // Set second client (queries)
        let receiver_query_client = RefCell::new(
            ReceiverQueryClient::new(GStdRemoting)
        );

        // Return an instance of the program with clientes 
        Self {
            receiver_client,
            receiver_query_client
        }
    }

    // services ...
}
```

With this, you set the client in the state of your contract, it helps to use less tokens in each call




## Using client in services

In your service (in this example is in "contract_service.rs") you have to import the necessary traits to use the clints, and import the "RefMut", it is used to handle the mutable reference of each client (This is because the program will pass the client to each service as a reference):

```rust
use sails_rs::{
    calls::{Call, Query}, 
    cell::RefMut, 
    prelude::*
};

use crate::clients::app_client::{
    traits::{
        Receiver, 
        QuerySvc as ReceiverQuery
    },
    ReceiverEvents
};

pub struct MyService<'a, ReceiverClient, ReceiverQueryClient> {
    pub receiver_contract_service: RefMut<'a, ReceiverClient>,
    pub receiver_conotract_query: RefMut<'a, ReceiverQueryClient>
}

#[service]
impl<'a, ReceiverClient, ReceiverQueryClient> MyService<'a, ReceiverClient, ReceiverQueryClient>
where 
    ReceiverClient: Receiver,
    ReceiverQueryClient: ReceiverQuery
{
    pub fn new(
        receiver_contract_service: RefMut<'a, ReceiverClient>,
        receiver_conotract_query: RefMut<'a, ReceiverQueryClient>
    ) -> Self {
        Self {
            receiver_contract_service,
            receiver_conotract_query
        }
    }

    // Command method as example
    pub async fn change_receiver_contract_string_value(&mut self, new_val: String, receiver_id: ActorId) -> String {
        let response_from_contract = self.receiver_contract_service
            .set_string_value(new_val)
            .send_recv(receiver_id)
            .await;

        let Ok(receiver_message) = response_from_contract else {
            return String::from("Error in response of contract");
        };
        
        let ReceiverEvents::StringValueChanged { new, old } = receiver_message else {
            return String::from("Incorrect answer of contract");
        };

        format!("new: {}, old: {}", new, old);
    }

    // Query method as example
    pub async fn string_value_from_receiver_contract_state(&self, receiver_id: ActorId) -> String {
        let response_from_contract = self.receiver_conotract_query
            .state_string_value()
            .recv(receiver_id)
            .await;

        let Ok(state_string_value) = response_from_contract else {
            return String::from("Error in response of contract");
        };

        format!("Response of contract: {}", state_string_value)
    }
}
```

As you can see, a lifetime ('a) and two traits (ReceiverClient and ReceiverQueryClient) are used. This is necessary because a struct cannot handle a reference by itself, so a lifetime must be set ([more information](https://doc.rust-lang.org/stable/book/ch10-03-lifetime-syntax.html#lifetime-annotations-in-struct-definitions)).

And the trait bounds ReceiverClient and ReceiverQueryClient are set to get as arguments the necesary clients ([more information](https://doc.rust-lang.org/stable/book/ch10-02-traits.html#trait-bound-syntax))




## Final step: pass client to each service

And finally, in the program file (lib.rs) you have to pass the necessary clients to each client (if any), using the previous examples, it would be as follows:

```rust
// ... more code

use services::contract_service::MyService;

#[program]
impl MyProgram {
    // program initializer with both clients
    pub fn new() -> Self {
        // Set first client 
        let receiver_client = RefCell::new(
            ReceiverClient::new(GStdRemoting)
        );

        // Set second client (queries)
        let receiver_query_client = RefCell::new(
            ReceiverQueryClient::new(GStdRemoting)
        );

        // Return an instance of the program with clientes 
        Self {
            receiver_client,
            receiver_query_client
        }
    }

    // Specify custom route
    #[route("MyService")]
    pub fn my_service_svc(&self) -> MyService<'_, ReceiverClient<GStdRemoting>, ReceiverQueryClient<GStdRemoting>>
        // Return an instance of the service 
        MyService::new(
            // pass each service as a mutable reference 
            self.receiver_client.borrow_mut(), 
            self.receiver_query_client.borrow_mut()
        )
    }
```