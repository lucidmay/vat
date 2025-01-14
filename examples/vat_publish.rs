use vat::package::Package;

fn main(){
    let i = Package::publish("dcc".to_string(), "test_message".to_string());
    dbg!(i);
}