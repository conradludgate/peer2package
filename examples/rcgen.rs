use rcgen::{DnType, ExtendedKeyUsagePurpose, GeneralSubtree, NameConstraints};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ca = rcgen::Certificate::from_params({
        let mut params = rcgen::CertificateParams::default();
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params
            .distinguished_name
            .push(DnType::CommonName, "ca.peer2package.conrad.cafe");
        params.name_constraints = Some(NameConstraints {
            permitted_subtrees: vec![GeneralSubtree::DnsName(
                "peer2package.conrad.cafe".to_owned(),
            )],
            excluded_subtrees: vec![],
        });
        params
    })?;

    let peer1 = rcgen::Certificate::from_params({
        let name = "peer1.peer2package.conrad.cafe".to_owned();
        let mut params = rcgen::CertificateParams::new(vec![name.clone()]);
        params.distinguished_name.push(DnType::CommonName, name);
        params.use_authority_key_identifier_extension = true;
        // params.key_usages.push(KeyUsagePurpose::DigitalSignature);
        params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ClientAuth);
        params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ServerAuth);
        params
    })?;
    let peer2 = rcgen::Certificate::from_params({
        let name = "peer2.peer2package.conrad.cafe".to_owned();
        let mut params = rcgen::CertificateParams::new(vec![name.clone()]);
        params.distinguished_name.push(DnType::CommonName, name);
        params.use_authority_key_identifier_extension = true;
        // params.key_usages.push(KeyUsagePurpose::DigitalSignature);
        params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ClientAuth);
        params
            .extended_key_usages
            .push(ExtendedKeyUsagePurpose::ServerAuth);
        params
    })?;

    std::fs::create_dir_all("certs")?;
    std::fs::write("certs/ca.pem", ca.serialize_pem()?)?;
    std::fs::write("certs/peer1.pem", peer1.serialize_pem_with_signer(&ca)?)?;
    std::fs::write("certs/peer2.pem", peer2.serialize_pem_with_signer(&ca)?)?;
    std::fs::write("certs/peer1.key.pem", peer1.serialize_private_key_pem())?;
    std::fs::write("certs/peer2.key.pem", peer2.serialize_private_key_pem())?;

    Ok(())
}
