use generic_storage::models::Person;
use generic_storage::serializer::{Bincode, Borsh, Json};
use generic_storage::storage::Storage;

fn test_person() -> Person {
    Person {
        name: "Alice".to_string(),
        age: 25,
    }
}

#[test]
fn test_borsh_save_load() {
    let person = test_person();
    let mut storage = Storage::new(Borsh);
    assert!(!storage.has_data());

    storage.save(&person).unwrap();
    assert!(storage.has_data());

    let loaded: Person = storage.load().unwrap();
    assert_eq!(person, loaded);
}

#[test]
fn test_bincode_save_load() {
    let person = test_person();
    let mut storage = Storage::new(Bincode);

    storage.save(&person).unwrap();
    let loaded: Person = storage.load().unwrap();
    assert_eq!(person, loaded);
}

#[test]
fn test_json_save_load() {
    let person = test_person();
    let mut storage = Storage::new(Json);

    storage.save(&person).unwrap();
    let loaded: Person = storage.load().unwrap();
    assert_eq!(person, loaded);
}

#[test]
fn test_load_empty_storage() {
    let storage: Storage<Person, Borsh> = Storage::new(Borsh);
    assert!(storage.load().is_err());
}

#[test]
fn test_overwrite_data() {
    let person1 = Person {
        name: "Bob".to_string(),
        age: 40,
    };
    let person2 = Person {
        name: "Charlie".to_string(),
        age: 35,
    };

    let mut storage = Storage::new(Json);
    storage.save(&person1).unwrap();
    storage.save(&person2).unwrap();

    let loaded: Person = storage.load().unwrap();
    assert_eq!(person2, loaded);
}

#[test]
fn test_all_serializers_produce_same_result() {
    let person = test_person();

    let mut borsh_storage = Storage::new(Borsh);
    let mut bincode_storage = Storage::new(Bincode);
    let mut json_storage = Storage::new(Json);

    borsh_storage.save(&person).unwrap();
    bincode_storage.save(&person).unwrap();
    json_storage.save(&person).unwrap();

    let from_borsh: Person = borsh_storage.load().unwrap();
    let from_bincode: Person = bincode_storage.load().unwrap();
    let from_json: Person = json_storage.load().unwrap();

    assert_eq!(from_borsh, person);
    assert_eq!(from_bincode, person);
    assert_eq!(from_json, person);
}
