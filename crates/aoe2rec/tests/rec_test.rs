use aoe2rec::Savegame;

#[test]
fn it_parses_recs_with_multiple_chapters() {
    let path = std::path::PathBuf::from("Fallingstar_vs_Eden_multichapter.aoe2record");
    let savegame = Savegame::from_file(&path).unwrap();
    let player1: String = savegame.chapters[0].zheader.game_settings.players[0]
        .name
        .clone()
        .into();
    assert_eq!(player1, "Fallingstar");
    assert_eq!(savegame.chapters.len(), 9);
    // Chapter 0 only had a duration of 650,702 ms, whereas total game duration is 2,692,367 ms
    assert_eq!(savegame.get_duration(), 2692367);
}

#[test]
fn test_get_duration_empty_chapters() {
    let savegame = Savegame { chapters: vec![] };
    assert_eq!(savegame.get_duration(), 0);
}

#[test]
fn it_parses_recs_with_version_64_3() {
    let path = std::path::PathBuf::from("beargwyn_vs_kamlesh.aoe2record");
    let savegame = Savegame::from_file(&path).unwrap();
    assert_eq!(64u16, savegame.chapters[0].zheader.version_major);
    assert_eq!(3u16, savegame.chapters[0].zheader.version_minor);
    let player1 = &savegame.chapters[0].zheader.game_settings.players[0];
    assert_eq!(String::from(player1.name.clone()), "Beargwyn");
}
