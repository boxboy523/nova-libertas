use std::{fs, path::Path};

use bevy::asset::{Asset, AssetPath, Handle};
use bevy_hui::prelude::{AssetLoadAdaptor, VerboseHtmlError, parse_template};

struct NoopAssetLoader;

impl AssetLoadAdaptor for NoopAssetLoader {
    fn load<'a, A: Asset>(&mut self, _path: impl Into<AssetPath<'a>>) -> Handle<A> {
        Handle::default()
    }
}

#[test]
fn split_ui_templates_parse() {
    let paths = std::iter::once("assets/ui/src/main.html".to_owned()).chain(
        fs::read_dir("assets/ui/src/components")
            .unwrap()
            .map(|entry| entry.unwrap().path().display().to_string()),
    );

    for path in paths {
        let input = fs::read(Path::new(&path)).unwrap();
        let (_, template) = parse_template::<VerboseHtmlError>(&input, &mut NoopAssetLoader)
            .unwrap_or_else(|error| panic!("failed to parse {path}: {error:?}"));
        assert_eq!(
            template.root.len(),
            1,
            "{path} must have exactly one root node"
        );
    }
}
