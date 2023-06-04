// gst-inspect-rs
// Copyright (c) 2023, Cesar Fabian Orccon Chipana <cfoch.fabian@gmail.com>
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this program; if not, see <http://www.gnu.org/licenses/>.
extern crate gstreamer as gst;

use crate::gst::prelude::Cast;
use crate::gst::prelude::GstObjectExt;
use crate::gst::prelude::PluginFeatureExt;
use crate::gst::prelude::PluginFeatureExtManual;
use crate::gst::prelude::StaticType;
use ansi_term::Color;
use clap::Arg;
use clap::Command;

const BRBLUE: Color = Color::RGB(97, 127, 166);
const PLUGIN_NAME_COLOR: Color = BRBLUE;
const ELEMENT_NAME_COLOR: Color = Color::Green;
const PROP_NAME_COLOR: Color = BRBLUE;
const HEADING_COLOR: Color = Color::Yellow;

fn print_element_list() {
    let registry = gst::Registry::get();
    let mut plugins = registry.plugins();

    plugins.sort_by(|p1, p2| p1.plugin_name().as_str().cmp(p2.plugin_name().as_str()));
    for plugin in &plugins {
        let mut features = registry.features_by_plugin(&plugin.plugin_name());

        features.sort_by(|f1, f2| f1.name().as_str().cmp(f2.name().as_str()));
        for feature in &features {
            if let Some(element_factory) = feature.downcast_ref::<gst::ElementFactory>() {
                println!(
                    "{}:  {}: {}",
                    PLUGIN_NAME_COLOR.paint(plugin.plugin_name().to_string()),
                    ELEMENT_NAME_COLOR.paint(element_factory.name().to_string()),
                    element_factory.longname()
                );
            }
        }
    }
}

fn get_rank_name(rank: gst::Rank) -> (&'static str, u32) {
    match rank {
        gst::Rank::None => ("none", 0),
        gst::Rank::Marginal => ("marginal", 64),
        gst::Rank::Secondary => ("secondary", 128),
        gst::Rank::Primary => ("primary", 256),
        _ => todo!(),
    }
}

fn print_property(name: &str, value: &str) {
    let formatted_name = format!("{:<25}", name);
    println!(" {}{}", PROP_NAME_COLOR.paint(formatted_name), value);
}

fn print_factory_details_info(factory: &gst::ElementFactory) {
    // FIXME: gst::PluginFeature::rank() should return int32, instead of Rank.
    let (rank_name, rank) = get_rank_name(factory.rank());
    println!("{}", HEADING_COLOR.paint("Factory details:"));
    print_property("Rank", &format!("{} ({})", rank_name, rank));
    print_property("Long name", factory.longname());
    print_property("Klass", factory.klass());
    print_property("Description", factory.description());
    print_property("Author", factory.author());
}

fn print_plugin_info(plugin: &gst::Plugin) {
    println!("{}", HEADING_COLOR.paint("Plugin details:"));
    print_property("Name", plugin.plugin_name().as_str());
    print_property("Description", plugin.description().as_str());
    print_property(
        "Filename",
        &plugin.filename().map_or("(null)".to_string(), |f| {
            f.into_os_string().into_string().unwrap()
        }),
    ); // FIXME: unwrap?
    print_property("Version", plugin.version().as_str());
    print_property("License", plugin.license().as_str());
    print_property("Source module", plugin.source().as_str());
    if let Some(release_date) = plugin.release_date_string() {
        // TODO: Hnandle YYYY-MM-DD, YYYY-MM-DDTHH:MHZ, YYYY-MM-DDTHH:MMZ or YYYY-MM-DD HH:MM (UTC)
        print_property("Source release date", release_date.as_str());
    }
    print_property("Binary package", plugin.package().as_str());
    print_property("Origin URL", plugin.origin().as_str());
}

fn print_element_info(feature: &gst::PluginFeature) -> i32 {
    let factory = feature.load();
    if factory.is_err() {
        println!("selement plugin couldn't be loaded");
        return -1;
    }

    let element_factory = factory
        .as_ref()
        .unwrap()
        .downcast_ref::<gst::ElementFactory>();
    assert!(!element_factory.is_none());

    let element = element_factory.unwrap().create_with_name(None);
    if element.is_err() {
        println!("couldn't construct element for some reason");
        return -1;
    }

    print_factory_details_info(element_factory.unwrap());
    println!();

    if let Some(plugin) = feature.plugin() {
        print_plugin_info(&plugin);
    }

    return 0;
}

fn print_feature_info(feature_name: &str) -> i32 {
    let registry = gst::Registry::get();

    let feature = registry.find_feature(feature_name, gst::ElementFactory::static_type());
    if feature.is_none() {
        println!("No such element or plugin '{}'", feature_name);
        return -1;
    }

    print_element_info(&feature.unwrap());

    return 0;
}

fn main() {
    let matches = Command::new("prog")
        .arg(Arg::new("ELEMENT-NAME | PLUGIN-NAME"))
        .get_matches();
    let mut st: i32 = 0;

    gst::init().unwrap();
    if let Some(fname) = matches.get_one::<String>("ELEMENT-NAME | PLUGIN-NAME") {
        st = print_feature_info(fname);
    } else {
        print_element_list();
    }

    std::process::exit(st);
}
