use lofty::{
    config::WriteOptions,
    file::TaggedFileExt,
    probe::Probe,
    read_from_path,
    tag::{Accessor, ItemKey, ItemValue, TagExt, TagItem, TagType},
};

#[test]
fn write_track_number() {
    // let file: &str = "src/test.mp3";
    // let probe = Probe::open(file).unwrap().guess_file_type().unwrap();
    // let file_type = &probe.file_type();
    // println!("fileType: {:?}", &file_type);

    let mut tag = lofty::tag::Tag::new(TagType::Id3v2);
    tag.insert(TagItem::new(
        ItemKey::from_key(TagType::Id3v2, "TRCK"),
        ItemValue::Text(String::from("1")),
    ));

    println!("{:?}", tag.track()); // None
    println!("{:?}\n", tag.track_total()); // Some(1)

    tag.insert(TagItem::new(
        ItemKey::from_key(TagType::Id3v2, "TRCK"),
        ItemValue::Text(String::from("1/14")),
    ));

    println!("{:?}", tag.track()); // None
    println!("{:?}\n", tag.track_total()); // None

    tag.save_to_path("src/blank.mp3", WriteOptions::new());

    let mp3_file = read_from_path("src/blank.mp3").unwrap();

    for tag_item in mp3_file.tags() {
        for tag in tag_item.items() {
            println!("Found tag: {:?}", tag);
        }
    }

}
