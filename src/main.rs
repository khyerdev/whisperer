mod vector;
mod sem;

use vector as vect;

fn main() {
    let string = "machine, i will cut you down, break you apart, splay the gore of your profaned form across the stars. i will grind you down until the very sparks cry for mercy. my hands shill relish, ending you, here, and, now!";
    println!("{string}");
    let bytes = vect::bytes_from_string(string);

    let encrypted = sem::encrypt(bytes);

    let obufuscated = encrypted.clone();
    {
        let obufuscated = vect::bytes_to_string(obufuscated);
        println!("{obufuscated}");
    }

    let decrypted = sem::decrypt(encrypted);
    let renewed = {
        let no_null = vect::remove_null(decrypted);
        vect::bytes_to_string(no_null)
    };
    println!("{renewed}");
    //test
}

