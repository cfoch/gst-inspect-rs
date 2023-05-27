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
use ansi_term::Color;

const BRBLUE: Color = Color::RGB(97, 127, 166);
const PLUGIN_NAME_COLOR: Color = BRBLUE;
const ELEMENT_NAME_COLOR: Color = Color::Green;

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

fn main() {
    gst::init().unwrap();

    print_element_list();
}
