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
use crate::gst::prelude::ElementExt;
use crate::gst::prelude::ElementExtManual;
use crate::gst::prelude::GstObjectExt;
use crate::gst::prelude::GstValueExt;
use crate::gst::prelude::ObjectExt;
use crate::gst::prelude::PluginFeatureExt;
use crate::gst::prelude::PluginFeatureExtManual;
use crate::gst::prelude::StaticType;
use crate::gst::prelude::URIHandlerExt;
use ansi_term::Color;
use clap::Arg;
use clap::Command;
use core::ops::ControlFlow;

const BRBLUE: Color = Color::RGB(97, 127, 166);
const PLUGIN_NAME_COLOR: Color = BRBLUE;
const ELEMENT_NAME_COLOR: Color = Color::Green;
const PROP_NAME_COLOR: Color = BRBLUE;
const PROP_VALUE_COLOR: Color = Color::Yellow;
const HEADING_COLOR: Color = Color::Yellow;
const DATA_TYPE_COLOR: Color = Color::Green;
const CHILD_LINK_COLOR: Color = Color::Purple;
const CAPS_TYPE_COLOR: Color = Color::Yellow;
const STRUCT_NAME_COLOR: Color = Color::Yellow;
const CAPS_FEATURE_COLOR: Color = Color::Green;
const FIELD_VALUE_COLOR: Color = BRBLUE;
const FIELD_NAME_COLOR: Color = Color::Cyan;
const PROP_ATTR_VALUE_COLOR: Color = Color::Cyan;

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

fn print_property(name: &str, value: &str, width: usize, indent: usize, colon: bool) {
    let formatted_name = PROP_NAME_COLOR.paint(format!("{:<width$}", name));
    let indent_str = " ".repeat(indent);
    let colon_str = if colon { ": " } else { "" };
    println!("{}{}{}{}", indent_str, formatted_name, colon_str, value);
}

fn print_property_details(name: &str, value: &str) {
    print_property(name, value, 25, 2, false);
}

fn print_factory_details_info(factory: &gst::ElementFactory) {
    // FIXME: gst::PluginFeature::rank() should return int32, instead of Rank.
    let (rank_name, rank) = get_rank_name(factory.rank());
    println!("{}", HEADING_COLOR.paint("Factory details:"));
    print_property_details("Rank", &format!("{} ({})", rank_name, rank));
    print_property_details("Long name", factory.longname());
    print_property_details("Klass", factory.klass());
    print_property_details("Description", factory.description());
    print_property_details("Author", factory.author());
    println!();
}

fn print_plugin_info(plugin: &gst::Plugin) {
    println!("{}", HEADING_COLOR.paint("Plugin details:"));
    print_property_details("Name", plugin.plugin_name().as_str());
    print_property_details("Description", plugin.description().as_str());
    print_property_details(
        "Filename",
        &plugin.filename().map_or("(null)".to_string(), |f| {
            f.into_os_string().into_string().unwrap()
        }),
    ); // FIXME: unwrap?
    print_property_details("Version", plugin.version().as_str());
    print_property_details("License", plugin.license().as_str());
    print_property_details("Source module", plugin.source().as_str());
    if let Some(release_date) = plugin.release_date_string() {
        // TODO: Hnandle YYYY-MM-DD, YYYY-MM-DDTHH:MHZ, YYYY-MM-DDTHH:MMZ or YYYY-MM-DD HH:MM (UTC)
        print_property_details("Source release date", release_date.as_str());
    }
    print_property_details("Binary package", plugin.package().as_str());
    print_property_details("Origin URL", plugin.origin().as_str());
    println!();
}

fn hierarchy_foreach<F>(type_: gst::glib::Type, foreach_func: &mut F)
where
    F: FnMut(gst::glib::Type),
{
    if let Some(parent) = type_.parent() {
        hierarchy_foreach(parent, foreach_func);
    }

    foreach_func(type_);
}

fn print_hierarchy(type_: gst::glib::Type) {
    let mut level = 0;
    let mut func = |cur_type: gst::glib::Type| {
        if level > 0 {
            print!("{}", "     ".repeat(level - 1));
            print!(" {}", CHILD_LINK_COLOR.paint("+----"));
        }
        println!("{}", DATA_TYPE_COLOR.paint(cur_type.name()));
        level += 1;
    };

    hierarchy_foreach(type_, &mut func);
    println!();
}

fn print_interfaces(type_: gst::glib::Type) {
    let interfaces = type_.interfaces();
    if interfaces.is_empty() {
        return;
    }

    println!("{}:", HEADING_COLOR.paint("Implemented Interfaces"));
    for iface in interfaces.as_slice() {
        println!("  {}", DATA_TYPE_COLOR.paint(iface.name()));
    }
    println!();
}

fn print_caps(caps: &gst::Caps) {
    let indent = " ".repeat(6);

    if caps.is_any() {
        println!("{}{}", indent, CAPS_TYPE_COLOR.paint("ANY"));
        return;
    }
    if caps.is_empty() {
        println!("{}{}", indent, CAPS_TYPE_COLOR.paint("EMPTY"));
        return;
    }

    for i in 0..caps.size() {
        if let Some(structure) = caps.structure(i) {
            match caps.features(i) {
                Some(f) if f.is_any() || !f.is_equal(&gst::CAPS_FEATURES_MEMORY_SYSTEM_MEMORY) => {
                    println!(
                        "{}{}({})",
                        indent,
                        STRUCT_NAME_COLOR.paint(structure.name().as_str()),
                        CAPS_FEATURE_COLOR.paint(f.to_string()),
                    );
                }
                _ => println!(
                    "{}{}",
                    indent,
                    STRUCT_NAME_COLOR.paint(structure.name().as_str())
                ),
            };
            structure.foreach(|q, v| {
                if let Ok(val) = v.serialize() {
                    let width = 23;
                    println!(
                        "{}: {}",
                        FIELD_NAME_COLOR.paint(format!("{:>width$}", q.as_str().to_string())),
                        FIELD_VALUE_COLOR.paint(val.as_str())
                    );
                }
                ControlFlow::Continue(())
            });
        }
    }
}

fn print_pad_templates_info(factory: &gst::ElementFactory) {
    let n_pads = factory.num_pad_templates();
    let indent = 2;

    println!("{}:", HEADING_COLOR.paint("Pad Templates"));
    if n_pads == 0 {
        println!(" none");
        return;
    }

    let mut pad_templates = factory.static_pad_templates().clone();
    pad_templates.sort_by(|t1, t2| t1.name_template().cmp(t2.name_template()));

    for pad_tmpl in pad_templates {
        let availability = match pad_tmpl.presence() {
            gst::PadPresence::Always => "Always",
            gst::PadPresence::Sometimes => "Sometimes",
            gst::PadPresence::Request => "On request",
            // FIXME?: gst::PadPresence::Unknown => "UNKNOWN",
        };

        print_property(
            &format!(
                "{} template",
                match pad_tmpl.direction() {
                    gst::PadDirection::Src => "SOURCE",
                    gst::PadDirection::Sink => "SINK",
                    gst::PadDirection::Unknown => "UNKNOWN",
                }
            ),
            &format!("'{}'", pad_tmpl.name_template()),
            0,
            indent,
            true,
        );
        print_property("Availability", availability, 0, indent * 2, true);
        print_property("Capabilities", "", 0, indent * 2, true);
        print_caps(&pad_tmpl.caps());
        println!();
    }
}

fn print_clocking_info(element: &gst::Element) {
    let flags = element.element_flags();
    let requires_clock = flags.intersects(gst::ElementFlags::REQUIRE_CLOCK);
    let provides_clock = flags.intersects(gst::ElementFlags::PROVIDE_CLOCK);

    if requires_clock || provides_clock {
        let indent = " ".repeat(2);

        println!();
        print_property("Clocking interaction", "", 0, 0, true);

        print!("{}", indent);
        if requires_clock {
            println!("{}", "element requires a clock");
        }
        if provides_clock {
            if let Some(clock) = element.clock() {
                println!(
                    "{}: {}",
                    PROP_VALUE_COLOR.paint("element provides a clock"),
                    DATA_TYPE_COLOR.paint(clock.name().as_str())
                );
            } else {
                println!(
                    "{}",
                    PROP_VALUE_COLOR
                        .paint("element is supposed to provide a clock but returned NULL")
                );
            }
        }
    } else {
        println!("Element has no clocking capabilities.");
    }
}

fn print_uri_handler_info(element: &gst::Element) {
    if let Some(uri_handler) = element.dynamic_cast_ref::<gst::URIHandler>() {
        let indent = " ".repeat(2);
        let uri_type = match uri_handler.uri_type() {
            gst::URIType::Src => "source",
            gst::URIType::Sink => "sink",
            gst::URIType::Unknown => "unknown",
        };
        println!();
        println!("{}", HEADING_COLOR.paint("URI handling capabilities:"));
        println!("{}Element can act as {}.", indent, uri_type);

        let uri_protocols = uri_handler.protocols();
        if uri_protocols.is_empty() {
            println!(
                "{}{}",
                indent,
                PROP_VALUE_COLOR.paint("No supported URI protocols")
            );
        } else {
            println!("{}Supported URI protocols:", indent);
        }
        uri_protocols.iter().for_each(|prot| {
            let indent = indent.repeat(2);
            println!("{}{}", indent, PROP_ATTR_VALUE_COLOR.paint(prot.as_str()));
        });
    } else {
        println!("Element has no URI handling capabilities.");
    }
    println!();
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
    if let Some(plugin) = feature.plugin() {
        print_plugin_info(&plugin);
    }
    let gtype = element.as_ref().unwrap().type_();
    print_hierarchy(gtype);
    print_interfaces(gtype);
    print_pad_templates_info(element_factory.unwrap());
    print_clocking_info(&element.as_ref().unwrap());
    print_uri_handler_info(&element.as_ref().unwrap());

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
