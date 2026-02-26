use generic_storage::models::Person;
use generic_storage::serializer::{Bincode, Borsh, Json};
use generic_storage::storage::Storage;

fn main() {
    let person = Person {
        name: "Andre".to_string(),
        age: 30,
    };

    println!("=== Borsh Serialization ===");
    let mut borsh_storage: Storage<Person, Borsh> = Storage::new(Borsh);
    println!("Has data before save: {}", borsh_storage.has_data());

    borsh_storage.save(&person).unwrap();
    println!("Has data after save: {}", borsh_storage.has_data());

    let loaded = borsh_storage.load().unwrap();
    println!("Loaded: {:?}", loaded);

    println!("\n=== Bincode Serialization ===");
    let mut bincode_storage: Storage<Person, Bincode> = Storage::new(Bincode);
    bincode_storage.save(&person).unwrap();
    let loaded = bincode_storage.load().unwrap();
    println!("Loaded: {:?}", loaded);

    println!("\n=== JSON Serialization ===");
    let mut json_storage: Storage<Person, Json> = Storage::new(Json);
    json_storage.save(&person).unwrap();
    let loaded = json_storage.load().unwrap();
    println!("Loaded: {:?}", loaded);

    println!("\nAll serialization formats work correctly!");
}
