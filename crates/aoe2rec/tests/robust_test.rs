use aoe2rec::{actions::ActionData, actions::Game, parse_operations, Operation, Savegame};
use binrw::BinReaderExt;
use std::io::Cursor;
use std::path::Path;

#[test]
fn test_parse_ai_operation() {
    let mut data = Cursor::new(vec![
        0x07, 0x00, 0x00, 0x00, // Magic 7 (AI)
        0x04, 0x00, 0x00, 0x00, // Length 4
        0xDE, 0xAD, 0xBE, 0xEF, // AI Data
    ]);
    let op: Operation = data.read_le_args((1u16,)).unwrap();
    if let Operation::Ai { length, data } = op {
        assert_eq!(length, 4);
        assert_eq!(data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    } else {
        panic!("Expected AI operation");
    }
}

#[test]
fn test_parse_operations_resync() {
    // parse_operations stops at magic 6.
    let mut data = Cursor::new(vec![
        0x99, 0x99, 0x99, // 3 bytes of garbage
        0x08, 0x00, 0x00, 0x00, // Magic 8 (MapNote)
        0x02, 0x00, 0x00, 0x00, // Length 2
        0x01, 0x02, // Data
        0x06, 0x00, 0x00, 0x00, // Magic 6 (PostGame)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, // Padding for PostGame seeks
        0xCE, 0xA4, 0x59, 0xB1, 0x05, 0xDB, 0x7B, 0x43, // End bit
    ]);

    let ops = parse_operations(&mut data, binrw::Endian::Little, (1u16,)).unwrap();
    assert_eq!(ops.len(), 2);
    if let Operation::MapNote { length, .. } = &ops[0] {
        assert_eq!(*length, 2);
    } else {
        panic!("Expected MapNote operation");
    }
}

#[test]
fn test_parse_action_with_failed_data() {
    // Action magic = 1
    // Action header: length (u32), data (Vec<u8>), world_time (u32)
    let mut data = Cursor::new(vec![
        0x01, 0x00, 0x00, 0x00, // Magic 1 (Action)
        0x04, 0x00, 0x00, 0x00, // Length 4
        0x00, 0x00, 0x00, 0x00, // Garbage action data
        0x00, 0x00, 0x00, 0x00, // World time
    ]);
    let op: Operation = data.read_le_args((1u16,)).unwrap();
    if let Operation::Action { action_data, .. } = op {
        assert!(action_data.is_none());
    } else {
        panic!("Expected Action operation");
    }
}

#[test]
fn test_parse_ai_research_action_without_selected_buildings() {
    let mut data = Cursor::new(vec![
        0x01, 0x00, 0x00, 0x00, // Magic 1 (Action)
        0x11, 0x00, 0x00, 0x00, // Payload length 17
        0x65, // Action type 101 (Research)
        0x02, // Player 2
        0x0d, 0x00, // Action length 13
        0x84, 0x0b, 0x00, 0x00, // Building ID 2948
        0x01, 0x00, // Selected 1
        0x16, 0x00, // Technology 22
        0xff, 0xff, 0xff, 0xff, 0x00, // Unknown trailer
        0xbd, 0x6a, 0x02, 0x00, // World time 158189
    ]);

    let op: Operation = data.read_le_args((66u16,)).unwrap();
    match op {
        Operation::Action {
            action_data:
                Some(ActionData::Research {
                    player_id,
                    action_length,
                    building_id,
                    selected,
                    technology_type,
                    building_ids,
                    ..
                }),
            ..
        } => {
            assert_eq!(player_id, 2);
            assert_eq!(action_length, 13);
            assert_eq!(building_id, 2948);
            assert_eq!(selected, 1);
            assert_eq!(technology_type, 22);
            assert!(building_ids.is_empty());
        }
        _ => panic!("Expected parsed AI research action"),
    }
}

#[test]
fn test_parse_research_action_with_selected_buildings() {
    let mut data = Cursor::new(vec![
        0x01, 0x00, 0x00, 0x00, // Magic 1 (Action)
        0x15, 0x00, 0x00, 0x00, // Payload length 21
        0x65, // Action type 101 (Research)
        0x01, // Player 1
        0x11, 0x00, // Action length 17
        0x7d, 0x0b, 0x00, 0x00, // Building ID 2941
        0x01, 0x00, // Selected 1
        0x16, 0x00, // Technology 22
        0xff, 0xff, 0xff, 0xff, 0x00, // Unknown trailer
        0x7d, 0x0b, 0x00, 0x00, // Selected building ID
        0xce, 0xbe, 0x07, 0x00, // World time 507470
    ]);

    let op: Operation = data.read_le_args((66u16,)).unwrap();
    match op {
        Operation::Action {
            action_data:
                Some(ActionData::Research {
                    player_id,
                    action_length,
                    building_id,
                    selected,
                    technology_type,
                    building_ids,
                    ..
                }),
            ..
        } => {
            assert_eq!(player_id, 1);
            assert_eq!(action_length, 17);
            assert_eq!(building_id, 2941);
            assert_eq!(selected, 1);
            assert_eq!(technology_type, 22);
            assert_eq!(building_ids, vec![2941]);
        }
        _ => panic!("Expected parsed research action with selected buildings"),
    }
}

#[test]
fn test_parse_game_cheat_action() {
    let mut data = Cursor::new(vec![
        0x01, 0x00, 0x00, 0x00, // Magic 1 (Action)
        0x14, 0x00, 0x00, 0x00, // Payload length 20
        0x67, // Action type 103 (Game)
        0x01, // Player 1
        0x10, 0x00, // Action length 16
        0x06, 0x00, 0x00, 0x00, // Game command 6 (Cheat)
        0x01, 0x00, // unknown1 = 1
        0x76, 0x00, // cheat_id = 118
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 8 bytes padding
        0xaa, 0x0f, 0x00, 0x00, // World time 4010
    ]);

    let op: Operation = data.read_le_args((66u16,)).unwrap();
    match op {
        Operation::Action {
            action_data:
                Some(ActionData::Game {
                    player_id,
                    action_length,
                    game_command: Game::Cheat { unknown1, cheat_id },
                }),
            world_time,
            ..
        } => {
            assert_eq!(player_id, 1);
            assert_eq!(action_length, 16);
            assert_eq!(unknown1, 1);
            assert_eq!(cheat_id, 118);
            assert_eq!(world_time, 4010);
        }
        _ => panic!("Expected parsed Game::Cheat action"),
    }
}

#[test]
fn test_parse_game_commands() {
    // Test FarmAutoqueue (16 / 0x10)
    let mut data = Cursor::new(vec![
        0x01, 0x00, 0x00, 0x00, // Magic 1 (Action)
        0x14, 0x00, 0x00, 0x00, // Payload length 20
        0x67, // Action type 103 (Game)
        0x02, // Player 2
        0x10, 0x00, // Action length 16
        0x10, 0x00, 0x00, 0x00, // Game command 16 (FarmAutoqueue)
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 12 bytes
        0x50, 0x00, 0x00, 0x00, // World time
    ]);

    let op: Operation = data.read_le_args((66u16,)).unwrap();
    match op {
        Operation::Action {
            action_data:
                Some(ActionData::Game {
                    player_id,
                    action_length,
                    game_command: Game::FarmAutoqueue {},
                }),
            ..
        } => {
            assert_eq!(player_id, 2);
            assert_eq!(action_length, 16);
        }
        _ => panic!("Expected parsed Game::FarmAutoqueue action"),
    }
}

#[test]
fn test_parse_cheats_recorded_game() {
    let path = Path::new("replay-cheats.aoe2record");
    let save = Savegame::from_file(path).unwrap();
    let cheats: Vec<_> = save
        .operations()
        .filter_map(|op| {
            if let Operation::Action {
                action_data:
                    Some(ActionData::Game {
                        game_command: Game::Cheat { unknown1, cheat_id },
                        ..
                    }),
                ..
            } = op
            {
                Some((*unknown1, *cheat_id))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(cheats.len(), 13);
    assert!(cheats.iter().all(|(_, id)| *id > 0));
    assert_eq!(cheats[0], (1, 100));
}

#[test]
fn test_parse_replay_aoe2record() {
    let path = Path::new("replay-ai.aoe2record");
    let save = Savegame::from_file(path).unwrap();
    assert_eq!(save.chapters[0].zheader.version_major, 67);
    assert_eq!(save.chapters[0].zheader.version_minor, 2);
    assert_eq!(save.operations().count(), 154047);
    assert_eq!(save.get_duration(), 2802638);
}

#[test]
fn test_parse_resume_aoe2record() {
    let path = Path::new("replay-resume.aoe2record");
    let save = Savegame::from_file(path).unwrap();
    assert_eq!(save.chapters[0].zheader.version_major, 68);
    assert_eq!(save.chapters[0].zheader.version_minor, 0);
    assert_eq!(save.operations().count(), 205277);
    assert_eq!(save.get_duration(), 3526015);
}

#[test]
fn test_load_shared_control_replay() {
    let path = Path::new("replay-shared.aoe2record");
    let res = Savegame::from_file(path);
    assert!(res.is_ok(), "Failed to parse shared-MP replay: {:?}", res.err());
    let save = res.unwrap();
    assert_eq!(save.get_duration(), 2161380);
    assert_eq!(save.operations().count(), 144051);
}
