use super::*;

#[test]
fn a_folder_name_is_trimmed_on_the_way_in() {
    assert_eq!(
        validate_folder_name("  Family facts  ").expect("trimmed"),
        "Family facts"
    );
}

#[test]
fn a_blank_folder_name_is_rejected() {
    for name in ["", "   ", "\t\n"] {
        let error = validate_folder_name(name).expect_err("blank name");
        assert_eq!(error.code, crate::errors::ErrorCode::BadRequest);
    }
}

#[test]
fn sixty_characters_is_allowed_and_sixty_one_is_not() {
    let sixty = "f".repeat(MAX_FOLDER_NAME_CHARS);
    assert_eq!(validate_folder_name(&sixty).expect("60 chars").len(), 60);

    let sixty_one = "f".repeat(MAX_FOLDER_NAME_CHARS + 1);
    let error = validate_folder_name(&sixty_one).expect_err("61 chars");
    assert_eq!(error.code, crate::errors::ErrorCode::BadRequest);
    assert!(error.message.contains("60"));
}

#[test]
fn the_bound_counts_characters_not_bytes() {
    // Thirty three-byte characters are ninety bytes but only thirty characters.
    let thirty_wide = "\u{4e00}".repeat(30);
    assert_eq!(thirty_wide.len(), 90);
    assert!(validate_folder_name(&thirty_wide).is_ok());
}

#[test]
fn the_default_folder_is_exempt_from_the_password_rule() {
    assert_eq!(
        validate_folder_password("", DEFAULT_FOLDER_ID).expect("exempt"),
        ""
    );
    assert_eq!(
        validate_folder_password("anything", DEFAULT_FOLDER_ID).expect("exempt"),
        ""
    );
    // The exemption is for that one id only.
    assert_eq!(
        validate_folder_password("", "fld-abc12345")
            .expect_err("protected folders need one")
            .code,
        crate::errors::ErrorCode::BadRequest
    );
    // Whitespace alone is not a password either.
    assert_eq!(
        validate_folder_password("   ", "fld-abc12345")
            .expect_err("blank is not a password")
            .code,
        crate::errors::ErrorCode::BadRequest
    );
    assert_eq!(
        validate_folder_password("hunter2", "fld-abc12345").expect("kept"),
        "hunter2"
    );
    // A protected folder's password is stored as entered (FR-068).
    assert_eq!(
        validate_folder_password(" spaced ", "fld-abc12345").expect("verbatim"),
        " spaced "
    );
}

#[test]
fn generated_folder_ids_have_the_documented_shape() {
    for _ in 0..32 {
        let id = generate_folder_id();
        assert!(id.starts_with("fld-"), "{id}");
        assert_eq!(id.len(), FOLDER_ID_PREFIX.len() + FOLDER_ID_BODY_CHARS);
        assert!(is_valid_folder_id(&id), "{id}");
    }
}

#[test]
fn only_the_documented_id_form_is_accepted_as_valid() {
    assert!(!is_valid_folder_id("default"));
    assert!(!is_valid_folder_id("fld-short"));
    assert!(!is_valid_folder_id("fld-ABCDEFGH"));
    assert!(!is_valid_folder_id("lst-abcd1234"));
}
