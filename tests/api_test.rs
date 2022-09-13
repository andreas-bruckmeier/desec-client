use desec_client::DeSecClient;

fn read_apikey() -> Option<String> {
    std::env::var("DESEC_API_TOKEN").ok()
}

fn read_domain() -> Option<String> {
    std::env::var("DESEC_DOMAIN").ok()
}

fn read_subname() -> Option<String> {
    std::env::var("DESEC_SUBNAME").ok()
}

#[test]
fn test_account_info() {
    if let Some(key) = read_apikey() {
        let client = DeSecClient::new(key.clone());
        let account_info = client.get_account_info();
        assert!(account_info.is_ok());
        assert!(account_info.unwrap().email.contains("@"));
    }
}

#[test]
fn test_rrset() {
    if let (Some(key), Some(domain), Some(subname)) 
            = (read_apikey(), read_domain(), read_subname()) {

        let client = DeSecClient::new(key.clone());
        let rrset_type = String::from("A");
        let records = vec!(String::from("8.8.8.8"));

        let rrset = client.create_rrset(
            domain.clone(),
            subname.clone(),
            rrset_type.clone(),
            records.clone(),
            3600
        );

        assert!(rrset.is_ok());
        assert_eq!(rrset.as_ref().unwrap().domain.clone().unwrap(), domain);
        assert_eq!(rrset.unwrap().records.unwrap(), records);

        std::thread::sleep(std::time::Duration::from_millis(1000));
        let rrset = client.get_rrset(
            &domain,
            &subname,
            &rrset_type
        );

        assert!(rrset.is_ok());
        let mut rrset = rrset.unwrap();

        assert_eq!(rrset.domain.clone().unwrap(), domain);
        assert_eq!(rrset.records.clone().unwrap(), records);

        rrset.ttl = Some(3650);
        
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let rrset = client.update_rrset(
            &domain,
            &subname,
            &rrset_type,
            &rrset
        );

        assert!(rrset.is_ok());
        let rrset = rrset.unwrap();

        assert_eq!(rrset.domain.clone().unwrap(), domain);
        assert_eq!(rrset.ttl.clone().unwrap(), 3650);

        std::thread::sleep(std::time::Duration::from_millis(1000));
        match client.delete_rrset(
            &domain,
            &subname,
            &rrset_type
        ) {
            Ok(_) => {},
            Err(err) => {
                println!("{:#?}", err);
            }
        }
    }
}
