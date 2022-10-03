use desec_client::DeSecClient;
use desec_client::DeSecError;
use tokio::time::{sleep, Duration};

fn read_apikey() -> Option<String> {
    std::env::var("DESEC_API_TOKEN").ok()
}

fn read_domain() -> Option<String> {
    std::env::var("DESEC_DOMAIN").ok()
}

fn read_domain_create() -> Option<String> {
    std::env::var("DESEC_DOMAIN_CREATE").ok()
}

fn read_subname() -> Option<String> {
    std::env::var("DESEC_SUBNAME").ok()
}

type Domain = String;
type Subname = String;

fn setup() -> (DeSecClient, Domain, Subname) {
    let key = read_apikey().unwrap();
    let domain = read_domain().unwrap();
    let subname = read_subname().unwrap();
    let client = DeSecClient::new(key).unwrap();
    (client, domain, subname)
}

#[tokio::test]
async fn create_delete_domain() {
    let (client, _, _) = setup();
    let domain_create = read_domain_create().unwrap();
    let sleep_duration = Duration::from_millis(1000);

    // check if domain exists
    match client.get_domain(&domain_create).await {
        // if domain exists, we try to delete it
        Ok(_) => {
            let res = client.delete_domain(&domain_create).await;
            assert!(res.is_ok(), "Failed to delete existing domain");
        }
        // nothing to do if domain does not exist
        Err(DeSecError::NotFound) => {}
        // we are unable to decide if the domain exists
        Err(_) => panic!("Could not check if domain already exists"),
    };

    sleep(sleep_duration).await;

    let result = client.create_domain(&domain_create).await;
    assert!(result.is_ok(), "Failed to create domain");

    sleep(sleep_duration).await;

    // We successfully created the domain, now lets clean up
    let res = client.delete_domain(&domain_create).await;
    assert!(res.is_ok(), "Failed to delete previously created domain");
}

#[tokio::test]
async fn create_update_delete_rrset() {
    let (client, domain, subname) = setup();

    let rrset_type = String::from("A");
    let records = vec![String::from("8.8.8.8")];
    let sleep_duration = Duration::from_millis(1000);

    // check if rrset exists
    match client.get_rrset(&domain, &subname, &rrset_type).await {
        // if rrset exists, we try to delete it
        Ok(_) => {
            let res = client.delete_rrset(&domain, &subname, &rrset_type).await;
            assert!(res.is_ok(), "Failed to delete existing rrset");
        }
        // nothing to do if rrset does not exist
        Err(DeSecError::NotFound) => {}
        // we are unable to decide if the domain exists
        Err(_) => panic!("Could not check if rrset already exists"),
    };

    sleep(sleep_duration).await;

    // create new rrset
    let result = client
        .create_rrset(
            domain.clone(),
            subname.clone(),
            rrset_type.clone(),
            records.clone(),
            3600,
        )
        .await;
    assert!(result.is_ok(), "Failed to create rrset");

    sleep(sleep_duration).await;

    // get new created rrset
    let result = client.get_rrset(&domain, &subname, &rrset_type).await;
    assert!(result.is_ok(), "Failed to get new rrset");

    sleep(sleep_duration).await;

    // update new rrset
    let mut rrset = result.unwrap();
    rrset.ttl = Some(3650);
    rrset.records = Some(vec![String::from("8.8.4.4")]);
    let rrset = client
        .update_rrset(&domain, &subname, &rrset_type, &rrset)
        .await;
    assert!(rrset.is_ok(), "Failed to update rrset");

    sleep(sleep_duration).await;

    let result = client.delete_rrset(&domain, &subname, &rrset_type).await;
    assert!(result.is_ok(), "Failed to delete rrset");
}

#[tokio::test]
async fn get_rrsets() {
    let (client, domain, _) = setup();
    let rrsets = client.get_rrsets(&domain).await;
    assert!(rrsets.is_ok(), "Failed to get rrsets");
    let rrsets = rrsets.unwrap();
    // at least one NS record should be in the result
    let rrset_type = "NS";
    let empty_string = String::new();
    assert!(
        rrsets
            .iter()
            .filter(|rrset| rrset.rrset_type.as_ref().unwrap_or(&empty_string) == rrset_type)
            .count()
            > 0
    );
}
