use crate::errors::{ApiError, ApiResult};
use rusqlite::{Connection, OptionalExtension, params};

pub const MIN_ITEMS: usize = 11;
pub const MAX_ITEMS: usize = 100;
pub const MAX_LIST_TITLE_CHARS: usize = 80;
pub const MAX_COLUMN_NAME_CHARS: usize = 40;
pub const MAX_LIST_DESCRIPTION_CHARS: usize = 200;

/// Timestamps of the seeded lists are fixed so a fresh database is reproducible.
const SEED_TIME: i64 = 1_710_000_000_000;

pub struct DefaultColumn {
    pub id: &'static str,
    pub name: &'static str,
}

pub struct DefaultItem {
    pub id: &'static str,
    pub name: &'static str,
    pub icon_name: &'static str,
    /// (column id, display text, numeric value)
    pub values: &'static [(&'static str, &'static str, f64)],
}

pub struct DefaultList {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub order_direction: &'static str,
    pub columns: &'static [DefaultColumn],
    pub items: &'static [DefaultItem],
}

const HISTORY_COLUMNS: &[DefaultColumn] = &[DefaultColumn {
    id: "year",
    name: "Year",
}];

const HISTORY_ITEMS: &[DefaultItem] = &[
    DefaultItem {
        id: "h_1",
        name: "Gutenberg Printing Press",
        icon_name: "BookOpen",
        values: &[("year", "1440", 1440.0)],
    },
    DefaultItem {
        id: "h_2",
        name: "Eiffel Tower Completed",
        icon_name: "Landmark",
        values: &[("year", "1889", 1889.0)],
    },
    DefaultItem {
        id: "h_3",
        name: "First Modern Olympic Games",
        icon_name: "Trophy",
        values: &[("year", "1896", 1896.0)],
    },
    DefaultItem {
        id: "h_4",
        name: "Wright Brothers First Flight",
        icon_name: "Plane",
        values: &[("year", "1903", 1903.0)],
    },
    DefaultItem {
        id: "h_5",
        name: "Sinking of the Titanic",
        icon_name: "Compass",
        values: &[("year", "1912", 1912.0)],
    },
    DefaultItem {
        id: "h_6",
        name: "Discovery of Penicillin",
        icon_name: "FlaskConical",
        values: &[("year", "1928", 1928.0)],
    },
    DefaultItem {
        id: "h_7",
        name: "Yuri Gagarin Enters Space",
        icon_name: "Orbit",
        values: &[("year", "1961", 1961.0)],
    },
    DefaultItem {
        id: "h_anchor",
        name: "Apollo 11 Moon Landing",
        icon_name: "Rocket",
        values: &[("year", "1969", 1969.0)],
    },
    DefaultItem {
        id: "h_8",
        name: "Fall of the Berlin Wall",
        icon_name: "Shield",
        values: &[("year", "1989", 1989.0)],
    },
    DefaultItem {
        id: "h_9",
        name: "Launch of Wikipedia",
        icon_name: "Globe",
        values: &[("year", "2001", 2001.0)],
    },
    DefaultItem {
        id: "h_10",
        name: "Original iPhone Announced",
        icon_name: "Smartphone",
        values: &[("year", "2007", 2007.0)],
    },
    DefaultItem {
        id: "h_11",
        name: "Fall of Constantinople",
        icon_name: "Shield",
        values: &[("year", "1453", 1453.0)],
    },
    DefaultItem {
        id: "h_12",
        name: "Columbus Reaches the Americas",
        icon_name: "Compass",
        values: &[("year", "1492", 1492.0)],
    },
    DefaultItem {
        id: "h_13",
        name: "Copernicus Publishes Heliocentrism",
        icon_name: "Orbit",
        values: &[("year", "1543", 1543.0)],
    },
    DefaultItem {
        id: "h_14",
        name: "Newton Publishes Principia",
        icon_name: "BookOpen",
        values: &[("year", "1687", 1687.0)],
    },
    DefaultItem {
        id: "h_15",
        name: "US Declaration of Independence",
        icon_name: "BookOpen",
        values: &[("year", "1776", 1776.0)],
    },
    DefaultItem {
        id: "h_16",
        name: "French Revolution Begins",
        icon_name: "Shield",
        values: &[("year", "1789", 1789.0)],
    },
    DefaultItem {
        id: "h_17",
        name: "First Steam Locomotive",
        icon_name: "Rocket",
        values: &[("year", "1804", 1804.0)],
    },
    DefaultItem {
        id: "h_18",
        name: "First Public Railway Opens",
        icon_name: "CableCar",
        values: &[("year", "1825", 1825.0)],
    },
    DefaultItem {
        id: "h_19",
        name: "Darwin Publishes On the Origin of Species",
        icon_name: "BookOpen",
        values: &[("year", "1859", 1859.0)],
    },
    DefaultItem {
        id: "h_20",
        name: "Suez Canal Opens",
        icon_name: "Compass",
        values: &[("year", "1869", 1869.0)],
    },
    DefaultItem {
        id: "h_21",
        name: "Telephone Patented",
        icon_name: "Radio",
        values: &[("year", "1876", 1876.0)],
    },
    DefaultItem {
        id: "h_22",
        name: "First Practical Light Bulb",
        icon_name: "Radio",
        values: &[("year", "1879", 1879.0)],
    },
    DefaultItem {
        id: "h_23",
        name: "First Transatlantic Radio Signal",
        icon_name: "Radio",
        values: &[("year", "1901", 1901.0)],
    },
    DefaultItem {
        id: "h_24",
        name: "Panama Canal Opens",
        icon_name: "Compass",
        values: &[("year", "1914", 1914.0)],
    },
    DefaultItem {
        id: "h_25",
        name: "First Television Demonstration",
        icon_name: "Radio",
        values: &[("year", "1926", 1926.0)],
    },
    DefaultItem {
        id: "h_26",
        name: "United Nations Founded",
        icon_name: "Globe",
        values: &[("year", "1945", 1945.0)],
    },
    DefaultItem {
        id: "h_27",
        name: "DNA Double Helix Discovered",
        icon_name: "FlaskConical",
        values: &[("year", "1953", 1953.0)],
    },
    DefaultItem {
        id: "h_28",
        name: "Sputnik Launched",
        icon_name: "Orbit",
        values: &[("year", "1957", 1957.0)],
    },
    DefaultItem {
        id: "h_29",
        name: "World Wide Web Goes Public",
        icon_name: "Globe",
        values: &[("year", "1991", 1991.0)],
    },
];

const STATUE_COLUMNS: &[DefaultColumn] = &[
    DefaultColumn {
        id: "height",
        name: "Height (m)",
    },
    DefaultColumn {
        id: "year_built",
        name: "Year Built",
    },
];

const STATUE_ITEMS: &[DefaultItem] = &[
    DefaultItem {
        id: "s_1",
        name: "Colosseum (Rome)",
        icon_name: "Landmark",
        values: &[("height", "48 m", 48.0), ("year_built", "80 AD", 80.0)],
    },
    DefaultItem {
        id: "s_2",
        name: "Statue of Liberty (New York)",
        icon_name: "Landmark",
        values: &[("height", "93 m", 93.0), ("year_built", "1886", 1886.0)],
    },
    DefaultItem {
        id: "s_3",
        name: "Big Ben Clock Tower (London)",
        icon_name: "Clock",
        values: &[("height", "96 m", 96.0), ("year_built", "1859", 1859.0)],
    },
    DefaultItem {
        id: "s_4",
        name: "Great Pyramid of Giza (Egypt)",
        icon_name: "Triangle",
        values: &[
            ("height", "138 m", 138.0),
            ("year_built", "2560 BC", -2560.0),
        ],
    },
    DefaultItem {
        id: "s_5",
        name: "Statue of Unity (India)",
        icon_name: "Landmark",
        values: &[("height", "182 m", 182.0), ("year_built", "2018", 2018.0)],
    },
    DefaultItem {
        id: "s_6",
        name: "Golden Gate Bridge Tower",
        icon_name: "CableCar",
        values: &[("height", "227 m", 227.0), ("year_built", "1937", 1937.0)],
    },
    DefaultItem {
        id: "s_anchor",
        name: "Eiffel Tower (Paris)",
        icon_name: "Landmark",
        values: &[("height", "330 m", 330.0), ("year_built", "1889", 1889.0)],
    },
    DefaultItem {
        id: "s_7",
        name: "Empire State Building (New York)",
        icon_name: "Building2",
        values: &[("height", "381 m", 381.0), ("year_built", "1931", 1931.0)],
    },
    DefaultItem {
        id: "s_8",
        name: "Taipei 101 (Taipei)",
        icon_name: "Building2",
        values: &[("height", "508 m", 508.0), ("year_built", "2004", 2004.0)],
    },
    DefaultItem {
        id: "s_9",
        name: "Tokyo Skytree (Tokyo)",
        icon_name: "Radio",
        values: &[("height", "634 m", 634.0), ("year_built", "2012", 2012.0)],
    },
    DefaultItem {
        id: "s_10",
        name: "Burj Khalifa (Dubai)",
        icon_name: "Building2",
        values: &[("height", "828 m", 828.0), ("year_built", "2010", 2010.0)],
    },
    DefaultItem {
        id: "s_11",
        name: "Leaning Tower of Pisa (Italy)",
        icon_name: "Landmark",
        values: &[("height", "56 m", 56.0), ("year_built", "1372", 1372.0)],
    },
    DefaultItem {
        id: "s_12",
        name: "Taj Mahal (India)",
        icon_name: "Landmark",
        values: &[("height", "73 m", 73.0), ("year_built", "1653", 1653.0)],
    },
    DefaultItem {
        id: "s_13",
        name: "Christ the Redeemer (Brazil)",
        icon_name: "Landmark",
        values: &[("height", "38 m", 38.0), ("year_built", "1931", 1931.0)],
    },
    DefaultItem {
        id: "s_14",
        name: "CN Tower (Canada)",
        icon_name: "Radio",
        values: &[("height", "553 m", 553.0), ("year_built", "1976", 1976.0)],
    },
    DefaultItem {
        id: "s_15",
        name: "Shanghai Tower (China)",
        icon_name: "Building2",
        values: &[("height", "632 m", 632.0), ("year_built", "2015", 2015.0)],
    },
    DefaultItem {
        id: "s_16",
        name: "Lotte World Tower (Seoul)",
        icon_name: "Building2",
        values: &[("height", "555 m", 555.0), ("year_built", "2017", 2017.0)],
    },
    DefaultItem {
        id: "s_17",
        name: "One World Trade Center (New York)",
        icon_name: "Building2",
        values: &[("height", "541 m", 541.0), ("year_built", "2014", 2014.0)],
    },
    DefaultItem {
        id: "s_18",
        name: "Petronas Towers (Kuala Lumpur)",
        icon_name: "Building2",
        values: &[("height", "452 m", 452.0), ("year_built", "1998", 1998.0)],
    },
    DefaultItem {
        id: "s_19",
        name: "Willis Tower (Chicago)",
        icon_name: "Building2",
        values: &[("height", "442 m", 442.0), ("year_built", "1974", 1974.0)],
    },
    DefaultItem {
        id: "s_20",
        name: "Mecca Royal Clock Tower (Mecca)",
        icon_name: "Clock",
        values: &[("height", "601 m", 601.0), ("year_built", "2012", 2012.0)],
    },
    DefaultItem {
        id: "s_21",
        name: "Merdeka 118 (Kuala Lumpur)",
        icon_name: "Building2",
        values: &[("height", "679 m", 679.0), ("year_built", "2023", 2023.0)],
    },
    DefaultItem {
        id: "s_22",
        name: "Cologne Cathedral (Germany)",
        icon_name: "Landmark",
        values: &[("height", "157 m", 157.0), ("year_built", "1880", 1880.0)],
    },
    DefaultItem {
        id: "s_23",
        name: "Washington Monument (USA)",
        icon_name: "Triangle",
        values: &[("height", "169 m", 169.0), ("year_built", "1884", 1884.0)],
    },
    DefaultItem {
        id: "s_24",
        name: "Sagrada Familia (Spain)",
        icon_name: "Landmark",
        values: &[("height", "172 m", 172.0), ("year_built", "1882", 1882.0)],
    },
];

const CINEMA_COLUMNS: &[DefaultColumn] = &[
    DefaultColumn {
        id: "box_office",
        name: "Box Office ($)",
    },
    DefaultColumn {
        id: "release_year",
        name: "Release Year",
    },
];

const CINEMA_ITEMS: &[DefaultItem] = &[
    DefaultItem {
        id: "m_1",
        name: "Pulp Fiction",
        icon_name: "Film",
        values: &[
            ("box_office", "$213 Million", 213.0),
            ("release_year", "1994", 1994.0),
        ],
    },
    DefaultItem {
        id: "m_2",
        name: "Star Wars: A New Hope",
        icon_name: "Film",
        values: &[
            ("box_office", "$775 Million", 775.0),
            ("release_year", "1977", 1977.0),
        ],
    },
    DefaultItem {
        id: "m_3",
        name: "The Lion King",
        icon_name: "Film",
        values: &[
            ("box_office", "$968 Million", 968.0),
            ("release_year", "1994", 1994.0),
        ],
    },
    DefaultItem {
        id: "m_4",
        name: "The Dark Knight",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.01 Billion", 1010.0),
            ("release_year", "2008", 2008.0),
        ],
    },
    DefaultItem {
        id: "m_5",
        name: "Jurassic Park",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.05 Billion", 1050.0),
            ("release_year", "1993", 1993.0),
        ],
    },
    DefaultItem {
        id: "m_6",
        name: "Barbie",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.44 Billion", 1440.0),
            ("release_year", "2023", 2023.0),
        ],
    },
    DefaultItem {
        id: "m_anchor",
        name: "Titanic",
        icon_name: "Film",
        values: &[
            ("box_office", "$2.26 Billion", 2260.0),
            ("release_year", "1997", 1997.0),
        ],
    },
    DefaultItem {
        id: "m_7",
        name: "Top Gun: Maverick",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.49 Billion", 1490.0),
            ("release_year", "2022", 2022.0),
        ],
    },
    DefaultItem {
        id: "m_8",
        name: "Spider-Man: No Way Home",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.92 Billion", 1920.0),
            ("release_year", "2021", 2021.0),
        ],
    },
    DefaultItem {
        id: "m_9",
        name: "Avengers: Endgame",
        icon_name: "Film",
        values: &[
            ("box_office", "$2.80 Billion", 2800.0),
            ("release_year", "2019", 2019.0),
        ],
    },
    DefaultItem {
        id: "m_10",
        name: "Avatar",
        icon_name: "Film",
        values: &[
            ("box_office", "$2.92 Billion", 2920.0),
            ("release_year", "2009", 2009.0),
        ],
    },
    DefaultItem {
        id: "m_11",
        name: "The Avengers",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.52 Billion", 1520.0),
            ("release_year", "2012", 2012.0),
        ],
    },
    DefaultItem {
        id: "m_12",
        name: "Frozen II",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.45 Billion", 1450.0),
            ("release_year", "2019", 2019.0),
        ],
    },
    DefaultItem {
        id: "m_13",
        name: "Avengers: Infinity War",
        icon_name: "Film",
        values: &[
            ("box_office", "$2.05 Billion", 2050.0),
            ("release_year", "2018", 2018.0),
        ],
    },
    DefaultItem {
        id: "m_14",
        name: "Star Wars: The Force Awakens",
        icon_name: "Film",
        values: &[
            ("box_office", "$2.07 Billion", 2070.0),
            ("release_year", "2015", 2015.0),
        ],
    },
    DefaultItem {
        id: "m_15",
        name: "The Super Mario Bros. Movie",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.36 Billion", 1360.0),
            ("release_year", "2023", 2023.0),
        ],
    },
    DefaultItem {
        id: "m_16",
        name: "Black Panther",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.35 Billion", 1350.0),
            ("release_year", "2018", 2018.0),
        ],
    },
    DefaultItem {
        id: "m_17",
        name: "Harry Potter and the Deathly Hallows - Part 2",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.34 Billion", 1340.0),
            ("release_year", "2011", 2011.0),
        ],
    },
    DefaultItem {
        id: "m_18",
        name: "Star Wars: The Last Jedi",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.33 Billion", 1330.0),
            ("release_year", "2017", 2017.0),
        ],
    },
    DefaultItem {
        id: "m_19",
        name: "Jurassic World",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.67 Billion", 1670.0),
            ("release_year", "2015", 2015.0),
        ],
    },
    DefaultItem {
        id: "m_20",
        name: "The Lion King (2019)",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.66 Billion", 1660.0),
            ("release_year", "2019", 2019.0),
        ],
    },
    DefaultItem {
        id: "m_21",
        name: "Furious 7",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.52 Billion", 1515.0),
            ("release_year", "2015", 2015.0),
        ],
    },
    DefaultItem {
        id: "m_22",
        name: "Frozen",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.29 Billion", 1290.0),
            ("release_year", "2013", 2013.0),
        ],
    },
    DefaultItem {
        id: "m_23",
        name: "Beauty and the Beast",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.27 Billion", 1266.0),
            ("release_year", "2017", 2017.0),
        ],
    },
    DefaultItem {
        id: "m_24",
        name: "Incredibles 2",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.24 Billion", 1243.0),
            ("release_year", "2018", 2018.0),
        ],
    },
    DefaultItem {
        id: "m_25",
        name: "The Fate of the Furious",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.24 Billion", 1236.0),
            ("release_year", "2017", 2017.0),
        ],
    },
    DefaultItem {
        id: "m_26",
        name: "Iron Man 3",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.21 Billion", 1215.0),
            ("release_year", "2013", 2013.0),
        ],
    },
    DefaultItem {
        id: "m_27",
        name: "Minions",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.16 Billion", 1159.0),
            ("release_year", "2015", 2015.0),
        ],
    },
    DefaultItem {
        id: "m_28",
        name: "Captain America: Civil War",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.15 Billion", 1153.0),
            ("release_year", "2016", 2016.0),
        ],
    },
    DefaultItem {
        id: "m_29",
        name: "Aquaman",
        icon_name: "Film",
        values: &[
            ("box_office", "$1.15 Billion", 1148.0),
            ("release_year", "2018", 2018.0),
        ],
    },
];

const COUNTRIES_COLUMNS: &[DefaultColumn] = &[
    DefaultColumn {
        id: "area_km2",
        name: "Area (km²)",
    },
    DefaultColumn {
        id: "population",
        name: "Population",
    },
    DefaultColumn {
        id: "founded",
        name: "Founded",
    },
];

/// Every sovereign state (193 UN members plus Vatican City and Palestine). Each
/// `icon_name` is `flag:<cc>` for the ISO 3166-1 alpha-2 code; `founded` is the
/// widely accepted independence/establishment year (negative for BC).
const COUNTRIES_ITEMS: &[DefaultItem] = &[
    DefaultItem {
        id: "c_af",
        name: "Afghanistan",
        icon_name: "flag:af",
        values: &[
            ("area_km2", "652,230", 652230.0),
            ("population", "41,000,000", 41000000.0),
            ("founded", "1919", 1919.0),
        ],
    },
    DefaultItem {
        id: "c_al",
        name: "Albania",
        icon_name: "flag:al",
        values: &[
            ("area_km2", "28,748", 28748.0),
            ("population", "2,800,000", 2800000.0),
            ("founded", "1912", 1912.0),
        ],
    },
    DefaultItem {
        id: "c_dz",
        name: "Algeria",
        icon_name: "flag:dz",
        values: &[
            ("area_km2", "2,381,741", 2381741.0),
            ("population", "45,600,000", 45600000.0),
            ("founded", "1962", 1962.0),
        ],
    },
    DefaultItem {
        id: "c_ad",
        name: "Andorra",
        icon_name: "flag:ad",
        values: &[
            ("area_km2", "468", 468.0),
            ("population", "80,000", 80000.0),
            ("founded", "1278", 1278.0),
        ],
    },
    DefaultItem {
        id: "c_ao",
        name: "Angola",
        icon_name: "flag:ao",
        values: &[
            ("area_km2", "1,246,700", 1246700.0),
            ("population", "36,600,000", 36600000.0),
            ("founded", "1975", 1975.0),
        ],
    },
    DefaultItem {
        id: "c_ag",
        name: "Antigua and Barbuda",
        icon_name: "flag:ag",
        values: &[
            ("area_km2", "442", 442.0),
            ("population", "94,000", 94000.0),
            ("founded", "1981", 1981.0),
        ],
    },
    DefaultItem {
        id: "c_ar",
        name: "Argentina",
        icon_name: "flag:ar",
        values: &[
            ("area_km2", "2,780,400", 2780400.0),
            ("population", "46,700,000", 46700000.0),
            ("founded", "1816", 1816.0),
        ],
    },
    DefaultItem {
        id: "c_am",
        name: "Armenia",
        icon_name: "flag:am",
        values: &[
            ("area_km2", "29,743", 29743.0),
            ("population", "3,000,000", 3000000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_au",
        name: "Australia",
        icon_name: "flag:au",
        values: &[
            ("area_km2", "7,692,024", 7692024.0),
            ("population", "26,600,000", 26600000.0),
            ("founded", "1901", 1901.0),
        ],
    },
    DefaultItem {
        id: "c_at",
        name: "Austria",
        icon_name: "flag:at",
        values: &[
            ("area_km2", "83,879", 83879.0),
            ("population", "9,100,000", 9100000.0),
            ("founded", "1918", 1918.0),
        ],
    },
    DefaultItem {
        id: "c_az",
        name: "Azerbaijan",
        icon_name: "flag:az",
        values: &[
            ("area_km2", "86,600", 86600.0),
            ("population", "10,200,000", 10200000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_bs",
        name: "Bahamas",
        icon_name: "flag:bs",
        values: &[
            ("area_km2", "13,943", 13943.0),
            ("population", "410,000", 410000.0),
            ("founded", "1973", 1973.0),
        ],
    },
    DefaultItem {
        id: "c_bh",
        name: "Bahrain",
        icon_name: "flag:bh",
        values: &[
            ("area_km2", "765", 765.0),
            ("population", "1,500,000", 1500000.0),
            ("founded", "1971", 1971.0),
        ],
    },
    DefaultItem {
        id: "c_bd",
        name: "Bangladesh",
        icon_name: "flag:bd",
        values: &[
            ("area_km2", "147,570", 147570.0),
            ("population", "173,000,000", 173000000.0),
            ("founded", "1971", 1971.0),
        ],
    },
    DefaultItem {
        id: "c_bb",
        name: "Barbados",
        icon_name: "flag:bb",
        values: &[
            ("area_km2", "430", 430.0),
            ("population", "280,000", 280000.0),
            ("founded", "1966", 1966.0),
        ],
    },
    DefaultItem {
        id: "c_by",
        name: "Belarus",
        icon_name: "flag:by",
        values: &[
            ("area_km2", "207,600", 207600.0),
            ("population", "9,200,000", 9200000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_be",
        name: "Belgium",
        icon_name: "flag:be",
        values: &[
            ("area_km2", "30,528", 30528.0),
            ("population", "11,800,000", 11800000.0),
            ("founded", "1830", 1830.0),
        ],
    },
    DefaultItem {
        id: "c_bz",
        name: "Belize",
        icon_name: "flag:bz",
        values: &[
            ("area_km2", "22,966", 22966.0),
            ("population", "410,000", 410000.0),
            ("founded", "1981", 1981.0),
        ],
    },
    DefaultItem {
        id: "c_bj",
        name: "Benin",
        icon_name: "flag:bj",
        values: &[
            ("area_km2", "114,763", 114763.0),
            ("population", "13,700,000", 13700000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_bt",
        name: "Bhutan",
        icon_name: "flag:bt",
        values: &[
            ("area_km2", "38,394", 38394.0),
            ("population", "780,000", 780000.0),
            ("founded", "1907", 1907.0),
        ],
    },
    DefaultItem {
        id: "c_bo",
        name: "Bolivia",
        icon_name: "flag:bo",
        values: &[
            ("area_km2", "1,098,581", 1098581.0),
            ("population", "12,400,000", 12400000.0),
            ("founded", "1825", 1825.0),
        ],
    },
    DefaultItem {
        id: "c_ba",
        name: "Bosnia and Herzegovina",
        icon_name: "flag:ba",
        values: &[
            ("area_km2", "51,197", 51197.0),
            ("population", "3,200,000", 3200000.0),
            ("founded", "1992", 1992.0),
        ],
    },
    DefaultItem {
        id: "c_bw",
        name: "Botswana",
        icon_name: "flag:bw",
        values: &[
            ("area_km2", "581,730", 581730.0),
            ("population", "2,600,000", 2600000.0),
            ("founded", "1966", 1966.0),
        ],
    },
    DefaultItem {
        id: "c_br",
        name: "Brazil",
        icon_name: "flag:br",
        values: &[
            ("area_km2", "8,515,767", 8515767.0),
            ("population", "216,000,000", 216000000.0),
            ("founded", "1822", 1822.0),
        ],
    },
    DefaultItem {
        id: "c_bn",
        name: "Brunei",
        icon_name: "flag:bn",
        values: &[
            ("area_km2", "5,765", 5765.0),
            ("population", "450,000", 450000.0),
            ("founded", "1984", 1984.0),
        ],
    },
    DefaultItem {
        id: "c_bg",
        name: "Bulgaria",
        icon_name: "flag:bg",
        values: &[
            ("area_km2", "110,879", 110879.0),
            ("population", "6,400,000", 6400000.0),
            ("founded", "1908", 1908.0),
        ],
    },
    DefaultItem {
        id: "c_bf",
        name: "Burkina Faso",
        icon_name: "flag:bf",
        values: &[
            ("area_km2", "272,967", 272967.0),
            ("population", "23,000,000", 23000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_bi",
        name: "Burundi",
        icon_name: "flag:bi",
        values: &[
            ("area_km2", "27,834", 27834.0),
            ("population", "13,200,000", 13200000.0),
            ("founded", "1962", 1962.0),
        ],
    },
    DefaultItem {
        id: "c_cv",
        name: "Cabo Verde",
        icon_name: "flag:cv",
        values: &[
            ("area_km2", "4,033", 4033.0),
            ("population", "600,000", 600000.0),
            ("founded", "1975", 1975.0),
        ],
    },
    DefaultItem {
        id: "c_kh",
        name: "Cambodia",
        icon_name: "flag:kh",
        values: &[
            ("area_km2", "181,035", 181035.0),
            ("population", "17,000,000", 17000000.0),
            ("founded", "1953", 1953.0),
        ],
    },
    DefaultItem {
        id: "c_cm",
        name: "Cameroon",
        icon_name: "flag:cm",
        values: &[
            ("area_km2", "475,442", 475442.0),
            ("population", "28,600,000", 28600000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_ca",
        name: "Canada",
        icon_name: "flag:ca",
        values: &[
            ("area_km2", "9,984,670", 9984670.0),
            ("population", "40,000,000", 40000000.0),
            ("founded", "1867", 1867.0),
        ],
    },
    DefaultItem {
        id: "c_cf",
        name: "Central African Republic",
        icon_name: "flag:cf",
        values: &[
            ("area_km2", "622,984", 622984.0),
            ("population", "5,700,000", 5700000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_td",
        name: "Chad",
        icon_name: "flag:td",
        values: &[
            ("area_km2", "1,284,000", 1284000.0),
            ("population", "18,000,000", 18000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_cl",
        name: "Chile",
        icon_name: "flag:cl",
        values: &[
            ("area_km2", "756,102", 756102.0),
            ("population", "19,600,000", 19600000.0),
            ("founded", "1818", 1818.0),
        ],
    },
    DefaultItem {
        id: "c_cn",
        name: "China",
        icon_name: "flag:cn",
        values: &[
            ("area_km2", "9,596,961", 9596961.0),
            ("population", "1,412,000,000", 1412000000.0),
            ("founded", "1949", 1949.0),
        ],
    },
    DefaultItem {
        id: "c_co",
        name: "Colombia",
        icon_name: "flag:co",
        values: &[
            ("area_km2", "1,141,748", 1141748.0),
            ("population", "52,200,000", 52200000.0),
            ("founded", "1810", 1810.0),
        ],
    },
    DefaultItem {
        id: "c_km",
        name: "Comoros",
        icon_name: "flag:km",
        values: &[
            ("area_km2", "1,862", 1862.0),
            ("population", "850,000", 850000.0),
            ("founded", "1975", 1975.0),
        ],
    },
    DefaultItem {
        id: "c_cg",
        name: "Congo (Republic of the)",
        icon_name: "flag:cg",
        values: &[
            ("area_km2", "342,000", 342000.0),
            ("population", "6,100,000", 6100000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_cd",
        name: "Congo (Democratic Republic of the)",
        icon_name: "flag:cd",
        values: &[
            ("area_km2", "2,344,858", 2344858.0),
            ("population", "102,000,000", 102000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_cr",
        name: "Costa Rica",
        icon_name: "flag:cr",
        values: &[
            ("area_km2", "51,100", 51100.0),
            ("population", "5,200,000", 5200000.0),
            ("founded", "1821", 1821.0),
        ],
    },
    DefaultItem {
        id: "c_ci",
        name: "Côte d'Ivoire",
        icon_name: "flag:ci",
        values: &[
            ("area_km2", "322,463", 322463.0),
            ("population", "28,900,000", 28900000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_hr",
        name: "Croatia",
        icon_name: "flag:hr",
        values: &[
            ("area_km2", "56,594", 56594.0),
            ("population", "3,800,000", 3800000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_cu",
        name: "Cuba",
        icon_name: "flag:cu",
        values: &[
            ("area_km2", "109,884", 109884.0),
            ("population", "11,000,000", 11000000.0),
            ("founded", "1902", 1902.0),
        ],
    },
    DefaultItem {
        id: "c_cy",
        name: "Cyprus",
        icon_name: "flag:cy",
        values: &[
            ("area_km2", "9,251", 9251.0),
            ("population", "1,300,000", 1300000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_cz",
        name: "Czechia",
        icon_name: "flag:cz",
        values: &[
            ("area_km2", "78,867", 78867.0),
            ("population", "10,900,000", 10900000.0),
            ("founded", "1993", 1993.0),
        ],
    },
    DefaultItem {
        id: "c_dk",
        name: "Denmark",
        icon_name: "flag:dk",
        values: &[
            ("area_km2", "43,094", 43094.0),
            ("population", "5,900,000", 5900000.0),
            ("founded", "1849", 1849.0),
        ],
    },
    DefaultItem {
        id: "c_dj",
        name: "Djibouti",
        icon_name: "flag:dj",
        values: &[
            ("area_km2", "23,200", 23200.0),
            ("population", "1,100,000", 1100000.0),
            ("founded", "1977", 1977.0),
        ],
    },
    DefaultItem {
        id: "c_dm",
        name: "Dominica",
        icon_name: "flag:dm",
        values: &[
            ("area_km2", "751", 751.0),
            ("population", "73,000", 73000.0),
            ("founded", "1978", 1978.0),
        ],
    },
    DefaultItem {
        id: "c_do",
        name: "Dominican Republic",
        icon_name: "flag:do",
        values: &[
            ("area_km2", "48,671", 48671.0),
            ("population", "11,300,000", 11300000.0),
            ("founded", "1844", 1844.0),
        ],
    },
    DefaultItem {
        id: "c_ec",
        name: "Ecuador",
        icon_name: "flag:ec",
        values: &[
            ("area_km2", "283,561", 283561.0),
            ("population", "18,000,000", 18000000.0),
            ("founded", "1822", 1822.0),
        ],
    },
    DefaultItem {
        id: "c_eg",
        name: "Egypt",
        icon_name: "flag:eg",
        values: &[
            ("area_km2", "1,002,450", 1002450.0),
            ("population", "112,000,000", 112000000.0),
            ("founded", "1922", 1922.0),
        ],
    },
    DefaultItem {
        id: "c_sv",
        name: "El Salvador",
        icon_name: "flag:sv",
        values: &[
            ("area_km2", "21,041", 21041.0),
            ("population", "6,300,000", 6300000.0),
            ("founded", "1821", 1821.0),
        ],
    },
    DefaultItem {
        id: "c_gq",
        name: "Equatorial Guinea",
        icon_name: "flag:gq",
        values: &[
            ("area_km2", "28,051", 28051.0),
            ("population", "1,700,000", 1700000.0),
            ("founded", "1968", 1968.0),
        ],
    },
    DefaultItem {
        id: "c_er",
        name: "Eritrea",
        icon_name: "flag:er",
        values: &[
            ("area_km2", "117,600", 117600.0),
            ("population", "3,700,000", 3700000.0),
            ("founded", "1993", 1993.0),
        ],
    },
    DefaultItem {
        id: "c_ee",
        name: "Estonia",
        icon_name: "flag:ee",
        values: &[
            ("area_km2", "45,227", 45227.0),
            ("population", "1,370,000", 1370000.0),
            ("founded", "1918", 1918.0),
        ],
    },
    DefaultItem {
        id: "c_sz",
        name: "Eswatini",
        icon_name: "flag:sz",
        values: &[
            ("area_km2", "17,364", 17364.0),
            ("population", "1,200,000", 1200000.0),
            ("founded", "1968", 1968.0),
        ],
    },
    DefaultItem {
        id: "c_et",
        name: "Ethiopia",
        icon_name: "flag:et",
        values: &[
            ("area_km2", "1,104,300", 1104300.0),
            ("population", "126,000,000", 126000000.0),
            ("founded", "1941", 1941.0),
        ],
    },
    DefaultItem {
        id: "c_fj",
        name: "Fiji",
        icon_name: "flag:fj",
        values: &[
            ("area_km2", "18,272", 18272.0),
            ("population", "930,000", 930000.0),
            ("founded", "1970", 1970.0),
        ],
    },
    DefaultItem {
        id: "c_fi",
        name: "Finland",
        icon_name: "flag:fi",
        values: &[
            ("area_km2", "338,424", 338424.0),
            ("population", "5,600,000", 5600000.0),
            ("founded", "1917", 1917.0),
        ],
    },
    DefaultItem {
        id: "c_fr",
        name: "France",
        icon_name: "flag:fr",
        values: &[
            ("area_km2", "551,695", 551695.0),
            ("population", "68,000,000", 68000000.0),
            ("founded", "1958", 1958.0),
        ],
    },
    DefaultItem {
        id: "c_ga",
        name: "Gabon",
        icon_name: "flag:ga",
        values: &[
            ("area_km2", "267,668", 267668.0),
            ("population", "2,400,000", 2400000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_gm",
        name: "Gambia",
        icon_name: "flag:gm",
        values: &[
            ("area_km2", "11,295", 11295.0),
            ("population", "2,700,000", 2700000.0),
            ("founded", "1965", 1965.0),
        ],
    },
    DefaultItem {
        id: "c_ge",
        name: "Georgia",
        icon_name: "flag:ge",
        values: &[
            ("area_km2", "69,700", 69700.0),
            ("population", "3,700,000", 3700000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_de",
        name: "Germany",
        icon_name: "flag:de",
        values: &[
            ("area_km2", "357,022", 357022.0),
            ("population", "84,000,000", 84000000.0),
            ("founded", "1990", 1990.0),
        ],
    },
    DefaultItem {
        id: "c_gh",
        name: "Ghana",
        icon_name: "flag:gh",
        values: &[
            ("area_km2", "238,533", 238533.0),
            ("population", "34,000,000", 34000000.0),
            ("founded", "1957", 1957.0),
        ],
    },
    DefaultItem {
        id: "c_gr",
        name: "Greece",
        icon_name: "flag:gr",
        values: &[
            ("area_km2", "131,957", 131957.0),
            ("population", "10,400,000", 10400000.0),
            ("founded", "1821", 1821.0),
        ],
    },
    DefaultItem {
        id: "c_gd",
        name: "Grenada",
        icon_name: "flag:gd",
        values: &[
            ("area_km2", "344", 344.0),
            ("population", "130,000", 130000.0),
            ("founded", "1974", 1974.0),
        ],
    },
    DefaultItem {
        id: "c_gt",
        name: "Guatemala",
        icon_name: "flag:gt",
        values: &[
            ("area_km2", "108,889", 108889.0),
            ("population", "18,000,000", 18000000.0),
            ("founded", "1821", 1821.0),
        ],
    },
    DefaultItem {
        id: "c_gn",
        name: "Guinea",
        icon_name: "flag:gn",
        values: &[
            ("area_km2", "245,857", 245857.0),
            ("population", "14,000,000", 14000000.0),
            ("founded", "1958", 1958.0),
        ],
    },
    DefaultItem {
        id: "c_gw",
        name: "Guinea-Bissau",
        icon_name: "flag:gw",
        values: &[
            ("area_km2", "36,125", 36125.0),
            ("population", "2,100,000", 2100000.0),
            ("founded", "1973", 1973.0),
        ],
    },
    DefaultItem {
        id: "c_gy",
        name: "Guyana",
        icon_name: "flag:gy",
        values: &[
            ("area_km2", "214,969", 214969.0),
            ("population", "810,000", 810000.0),
            ("founded", "1966", 1966.0),
        ],
    },
    DefaultItem {
        id: "c_ht",
        name: "Haiti",
        icon_name: "flag:ht",
        values: &[
            ("area_km2", "27,750", 27750.0),
            ("population", "11,700,000", 11700000.0),
            ("founded", "1804", 1804.0),
        ],
    },
    DefaultItem {
        id: "c_hn",
        name: "Honduras",
        icon_name: "flag:hn",
        values: &[
            ("area_km2", "112,492", 112492.0),
            ("population", "10,500,000", 10500000.0),
            ("founded", "1821", 1821.0),
        ],
    },
    DefaultItem {
        id: "c_hu",
        name: "Hungary",
        icon_name: "flag:hu",
        values: &[
            ("area_km2", "93,028", 93028.0),
            ("population", "9,600,000", 9600000.0),
            ("founded", "1918", 1918.0),
        ],
    },
    DefaultItem {
        id: "c_is",
        name: "Iceland",
        icon_name: "flag:is",
        values: &[
            ("area_km2", "103,000", 103000.0),
            ("population", "390,000", 390000.0),
            ("founded", "1944", 1944.0),
        ],
    },
    DefaultItem {
        id: "c_in",
        name: "India",
        icon_name: "flag:in",
        values: &[
            ("area_km2", "3,287,263", 3287263.0),
            ("population", "1,428,000,000", 1428000000.0),
            ("founded", "1947", 1947.0),
        ],
    },
    DefaultItem {
        id: "c_id",
        name: "Indonesia",
        icon_name: "flag:id",
        values: &[
            ("area_km2", "1,904,569", 1904569.0),
            ("population", "278,000,000", 278000000.0),
            ("founded", "1945", 1945.0),
        ],
    },
    DefaultItem {
        id: "c_ir",
        name: "Iran",
        icon_name: "flag:ir",
        values: &[
            ("area_km2", "1,648,195", 1648195.0),
            ("population", "89,000,000", 89000000.0),
            ("founded", "1979", 1979.0),
        ],
    },
    DefaultItem {
        id: "c_iq",
        name: "Iraq",
        icon_name: "flag:iq",
        values: &[
            ("area_km2", "438,317", 438317.0),
            ("population", "45,000,000", 45000000.0),
            ("founded", "1932", 1932.0),
        ],
    },
    DefaultItem {
        id: "c_ie",
        name: "Ireland",
        icon_name: "flag:ie",
        values: &[
            ("area_km2", "70,273", 70273.0),
            ("population", "5,300,000", 5300000.0),
            ("founded", "1922", 1922.0),
        ],
    },
    DefaultItem {
        id: "c_il",
        name: "Israel",
        icon_name: "flag:il",
        values: &[
            ("area_km2", "20,770", 20770.0),
            ("population", "9,800,000", 9800000.0),
            ("founded", "1948", 1948.0),
        ],
    },
    DefaultItem {
        id: "c_it",
        name: "Italy",
        icon_name: "flag:it",
        values: &[
            ("area_km2", "301,340", 301340.0),
            ("population", "59,000,000", 59000000.0),
            ("founded", "1861", 1861.0),
        ],
    },
    DefaultItem {
        id: "c_jm",
        name: "Jamaica",
        icon_name: "flag:jm",
        values: &[
            ("area_km2", "10,991", 10991.0),
            ("population", "2,800,000", 2800000.0),
            ("founded", "1962", 1962.0),
        ],
    },
    DefaultItem {
        id: "c_jp",
        name: "Japan",
        icon_name: "flag:jp",
        values: &[
            ("area_km2", "377,975", 377975.0),
            ("population", "124,000,000", 124000000.0),
            ("founded", "-660", -660.0),
        ],
    },
    DefaultItem {
        id: "c_jo",
        name: "Jordan",
        icon_name: "flag:jo",
        values: &[
            ("area_km2", "89,342", 89342.0),
            ("population", "11,300,000", 11300000.0),
            ("founded", "1946", 1946.0),
        ],
    },
    DefaultItem {
        id: "c_kz",
        name: "Kazakhstan",
        icon_name: "flag:kz",
        values: &[
            ("area_km2", "2,724,900", 2724900.0),
            ("population", "20,000,000", 20000000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_ke",
        name: "Kenya",
        icon_name: "flag:ke",
        values: &[
            ("area_km2", "580,367", 580367.0),
            ("population", "55,000,000", 55000000.0),
            ("founded", "1963", 1963.0),
        ],
    },
    DefaultItem {
        id: "c_ki",
        name: "Kiribati",
        icon_name: "flag:ki",
        values: &[
            ("area_km2", "811", 811.0),
            ("population", "130,000", 130000.0),
            ("founded", "1979", 1979.0),
        ],
    },
    DefaultItem {
        id: "c_kw",
        name: "Kuwait",
        icon_name: "flag:kw",
        values: &[
            ("area_km2", "17,818", 17818.0),
            ("population", "4,300,000", 4300000.0),
            ("founded", "1961", 1961.0),
        ],
    },
    DefaultItem {
        id: "c_kg",
        name: "Kyrgyzstan",
        icon_name: "flag:kg",
        values: &[
            ("area_km2", "199,951", 199951.0),
            ("population", "7,000,000", 7000000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_la",
        name: "Laos",
        icon_name: "flag:la",
        values: &[
            ("area_km2", "236,800", 236800.0),
            ("population", "7,600,000", 7600000.0),
            ("founded", "1953", 1953.0),
        ],
    },
    DefaultItem {
        id: "c_lv",
        name: "Latvia",
        icon_name: "flag:lv",
        values: &[
            ("area_km2", "64,589", 64589.0),
            ("population", "1,900,000", 1900000.0),
            ("founded", "1918", 1918.0),
        ],
    },
    DefaultItem {
        id: "c_lb",
        name: "Lebanon",
        icon_name: "flag:lb",
        values: &[
            ("area_km2", "10,452", 10452.0),
            ("population", "5,500,000", 5500000.0),
            ("founded", "1943", 1943.0),
        ],
    },
    DefaultItem {
        id: "c_ls",
        name: "Lesotho",
        icon_name: "flag:ls",
        values: &[
            ("area_km2", "30,355", 30355.0),
            ("population", "2,300,000", 2300000.0),
            ("founded", "1966", 1966.0),
        ],
    },
    DefaultItem {
        id: "c_lr",
        name: "Liberia",
        icon_name: "flag:lr",
        values: &[
            ("area_km2", "111,369", 111369.0),
            ("population", "5,400,000", 5400000.0),
            ("founded", "1847", 1847.0),
        ],
    },
    DefaultItem {
        id: "c_ly",
        name: "Libya",
        icon_name: "flag:ly",
        values: &[
            ("area_km2", "1,759,540", 1759540.0),
            ("population", "6,900,000", 6900000.0),
            ("founded", "1951", 1951.0),
        ],
    },
    DefaultItem {
        id: "c_li",
        name: "Liechtenstein",
        icon_name: "flag:li",
        values: &[
            ("area_km2", "160", 160.0),
            ("population", "40,000", 40000.0),
            ("founded", "1806", 1806.0),
        ],
    },
    DefaultItem {
        id: "c_lt",
        name: "Lithuania",
        icon_name: "flag:lt",
        values: &[
            ("area_km2", "65,300", 65300.0),
            ("population", "2,800,000", 2800000.0),
            ("founded", "1918", 1918.0),
        ],
    },
    DefaultItem {
        id: "c_lu",
        name: "Luxembourg",
        icon_name: "flag:lu",
        values: &[
            ("area_km2", "2,586", 2586.0),
            ("population", "660,000", 660000.0),
            ("founded", "1815", 1815.0),
        ],
    },
    DefaultItem {
        id: "c_mg",
        name: "Madagascar",
        icon_name: "flag:mg",
        values: &[
            ("area_km2", "587,041", 587041.0),
            ("population", "30,000,000", 30000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_mw",
        name: "Malawi",
        icon_name: "flag:mw",
        values: &[
            ("area_km2", "118,484", 118484.0),
            ("population", "21,000,000", 21000000.0),
            ("founded", "1964", 1964.0),
        ],
    },
    DefaultItem {
        id: "c_my",
        name: "Malaysia",
        icon_name: "flag:my",
        values: &[
            ("area_km2", "330,803", 330803.0),
            ("population", "34,000,000", 34000000.0),
            ("founded", "1957", 1957.0),
        ],
    },
    DefaultItem {
        id: "c_mv",
        name: "Maldives",
        icon_name: "flag:mv",
        values: &[
            ("area_km2", "300", 300.0),
            ("population", "520,000", 520000.0),
            ("founded", "1965", 1965.0),
        ],
    },
    DefaultItem {
        id: "c_ml",
        name: "Mali",
        icon_name: "flag:ml",
        values: &[
            ("area_km2", "1,240,192", 1240192.0),
            ("population", "23,000,000", 23000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_mt",
        name: "Malta",
        icon_name: "flag:mt",
        values: &[
            ("area_km2", "316", 316.0),
            ("population", "540,000", 540000.0),
            ("founded", "1964", 1964.0),
        ],
    },
    DefaultItem {
        id: "c_mh",
        name: "Marshall Islands",
        icon_name: "flag:mh",
        values: &[
            ("area_km2", "181", 181.0),
            ("population", "42,000", 42000.0),
            ("founded", "1979", 1979.0),
        ],
    },
    DefaultItem {
        id: "c_mr",
        name: "Mauritania",
        icon_name: "flag:mr",
        values: &[
            ("area_km2", "1,030,700", 1030700.0),
            ("population", "4,900,000", 4900000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_mu",
        name: "Mauritius",
        icon_name: "flag:mu",
        values: &[
            ("area_km2", "2,040", 2040.0),
            ("population", "1,300,000", 1300000.0),
            ("founded", "1968", 1968.0),
        ],
    },
    DefaultItem {
        id: "c_mx",
        name: "Mexico",
        icon_name: "flag:mx",
        values: &[
            ("area_km2", "1,964,375", 1964375.0),
            ("population", "129,000,000", 129000000.0),
            ("founded", "1810", 1810.0),
        ],
    },
    DefaultItem {
        id: "c_fm",
        name: "Micronesia",
        icon_name: "flag:fm",
        values: &[
            ("area_km2", "702", 702.0),
            ("population", "110,000", 110000.0),
            ("founded", "1979", 1979.0),
        ],
    },
    DefaultItem {
        id: "c_md",
        name: "Moldova",
        icon_name: "flag:md",
        values: &[
            ("area_km2", "33,846", 33846.0),
            ("population", "2,500,000", 2500000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_mc",
        name: "Monaco",
        icon_name: "flag:mc",
        values: &[
            ("area_km2", "2", 2.0),
            ("population", "39,000", 39000.0),
            ("founded", "1297", 1297.0),
        ],
    },
    DefaultItem {
        id: "c_mn",
        name: "Mongolia",
        icon_name: "flag:mn",
        values: &[
            ("area_km2", "1,564,110", 1564110.0),
            ("population", "3,400,000", 3400000.0),
            ("founded", "1911", 1911.0),
        ],
    },
    DefaultItem {
        id: "c_me",
        name: "Montenegro",
        icon_name: "flag:me",
        values: &[
            ("area_km2", "13,812", 13812.0),
            ("population", "620,000", 620000.0),
            ("founded", "2006", 2006.0),
        ],
    },
    DefaultItem {
        id: "c_ma",
        name: "Morocco",
        icon_name: "flag:ma",
        values: &[
            ("area_km2", "446,550", 446550.0),
            ("population", "37,500,000", 37500000.0),
            ("founded", "1956", 1956.0),
        ],
    },
    DefaultItem {
        id: "c_mz",
        name: "Mozambique",
        icon_name: "flag:mz",
        values: &[
            ("area_km2", "801,590", 801590.0),
            ("population", "33,000,000", 33000000.0),
            ("founded", "1975", 1975.0),
        ],
    },
    DefaultItem {
        id: "c_mm",
        name: "Myanmar",
        icon_name: "flag:mm",
        values: &[
            ("area_km2", "676,578", 676578.0),
            ("population", "54,000,000", 54000000.0),
            ("founded", "1948", 1948.0),
        ],
    },
    DefaultItem {
        id: "c_na",
        name: "Namibia",
        icon_name: "flag:na",
        values: &[
            ("area_km2", "825,615", 825615.0),
            ("population", "2,600,000", 2600000.0),
            ("founded", "1990", 1990.0),
        ],
    },
    DefaultItem {
        id: "c_nr",
        name: "Nauru",
        icon_name: "flag:nr",
        values: &[
            ("area_km2", "21", 21.0),
            ("population", "12,000", 12000.0),
            ("founded", "1968", 1968.0),
        ],
    },
    DefaultItem {
        id: "c_np",
        name: "Nepal",
        icon_name: "flag:np",
        values: &[
            ("area_km2", "147,181", 147181.0),
            ("population", "30,000,000", 30000000.0),
            ("founded", "1768", 1768.0),
        ],
    },
    DefaultItem {
        id: "c_nl",
        name: "Netherlands",
        icon_name: "flag:nl",
        values: &[
            ("area_km2", "41,850", 41850.0),
            ("population", "17,800,000", 17800000.0),
            ("founded", "1815", 1815.0),
        ],
    },
    DefaultItem {
        id: "c_nz",
        name: "New Zealand",
        icon_name: "flag:nz",
        values: &[
            ("area_km2", "268,021", 268021.0),
            ("population", "5,200,000", 5200000.0),
            ("founded", "1907", 1907.0),
        ],
    },
    DefaultItem {
        id: "c_ni",
        name: "Nicaragua",
        icon_name: "flag:ni",
        values: &[
            ("area_km2", "130,373", 130373.0),
            ("population", "6,900,000", 6900000.0),
            ("founded", "1821", 1821.0),
        ],
    },
    DefaultItem {
        id: "c_ne",
        name: "Niger",
        icon_name: "flag:ne",
        values: &[
            ("area_km2", "1,267,000", 1267000.0),
            ("population", "27,000,000", 27000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_ng",
        name: "Nigeria",
        icon_name: "flag:ng",
        values: &[
            ("area_km2", "923,768", 923768.0),
            ("population", "223,000,000", 223000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_kp",
        name: "North Korea",
        icon_name: "flag:kp",
        values: &[
            ("area_km2", "120,538", 120538.0),
            ("population", "26,000,000", 26000000.0),
            ("founded", "1948", 1948.0),
        ],
    },
    DefaultItem {
        id: "c_mk",
        name: "North Macedonia",
        icon_name: "flag:mk",
        values: &[
            ("area_km2", "25,713", 25713.0),
            ("population", "1,800,000", 1800000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_no",
        name: "Norway",
        icon_name: "flag:no",
        values: &[
            ("area_km2", "385,207", 385207.0),
            ("population", "5,500,000", 5500000.0),
            ("founded", "1905", 1905.0),
        ],
    },
    DefaultItem {
        id: "c_om",
        name: "Oman",
        icon_name: "flag:om",
        values: &[
            ("area_km2", "309,500", 309500.0),
            ("population", "4,600,000", 4600000.0),
            ("founded", "1650", 1650.0),
        ],
    },
    DefaultItem {
        id: "c_pk",
        name: "Pakistan",
        icon_name: "flag:pk",
        values: &[
            ("area_km2", "881,913", 881913.0),
            ("population", "240,000,000", 240000000.0),
            ("founded", "1947", 1947.0),
        ],
    },
    DefaultItem {
        id: "c_pw",
        name: "Palau",
        icon_name: "flag:pw",
        values: &[
            ("area_km2", "459", 459.0),
            ("population", "18,000", 18000.0),
            ("founded", "1981", 1981.0),
        ],
    },
    DefaultItem {
        id: "c_ps",
        name: "Palestine",
        icon_name: "flag:ps",
        values: &[
            ("area_km2", "6,020", 6020.0),
            ("population", "5,400,000", 5400000.0),
            ("founded", "1988", 1988.0),
        ],
    },
    DefaultItem {
        id: "c_pa",
        name: "Panama",
        icon_name: "flag:pa",
        values: &[
            ("area_km2", "75,417", 75417.0),
            ("population", "4,400,000", 4400000.0),
            ("founded", "1903", 1903.0),
        ],
    },
    DefaultItem {
        id: "c_pg",
        name: "Papua New Guinea",
        icon_name: "flag:pg",
        values: &[
            ("area_km2", "462,840", 462840.0),
            ("population", "10,300,000", 10300000.0),
            ("founded", "1975", 1975.0),
        ],
    },
    DefaultItem {
        id: "c_py",
        name: "Paraguay",
        icon_name: "flag:py",
        values: &[
            ("area_km2", "406,752", 406752.0),
            ("population", "6,800,000", 6800000.0),
            ("founded", "1811", 1811.0),
        ],
    },
    DefaultItem {
        id: "c_pe",
        name: "Peru",
        icon_name: "flag:pe",
        values: &[
            ("area_km2", "1,285,216", 1285216.0),
            ("population", "34,000,000", 34000000.0),
            ("founded", "1821", 1821.0),
        ],
    },
    DefaultItem {
        id: "c_ph",
        name: "Philippines",
        icon_name: "flag:ph",
        values: &[
            ("area_km2", "300,000", 300000.0),
            ("population", "117,000,000", 117000000.0),
            ("founded", "1946", 1946.0),
        ],
    },
    DefaultItem {
        id: "c_pl",
        name: "Poland",
        icon_name: "flag:pl",
        values: &[
            ("area_km2", "312,696", 312696.0),
            ("population", "38,000,000", 38000000.0),
            ("founded", "1918", 1918.0),
        ],
    },
    DefaultItem {
        id: "c_pt",
        name: "Portugal",
        icon_name: "flag:pt",
        values: &[
            ("area_km2", "92,212", 92212.0),
            ("population", "10,400,000", 10400000.0),
            ("founded", "1143", 1143.0),
        ],
    },
    DefaultItem {
        id: "c_qa",
        name: "Qatar",
        icon_name: "flag:qa",
        values: &[
            ("area_km2", "11,586", 11586.0),
            ("population", "2,700,000", 2700000.0),
            ("founded", "1971", 1971.0),
        ],
    },
    DefaultItem {
        id: "c_ro",
        name: "Romania",
        icon_name: "flag:ro",
        values: &[
            ("area_km2", "238,397", 238397.0),
            ("population", "19,000,000", 19000000.0),
            ("founded", "1877", 1877.0),
        ],
    },
    DefaultItem {
        id: "c_ru",
        name: "Russia",
        icon_name: "flag:ru",
        values: &[
            ("area_km2", "17,098,246", 17098246.0),
            ("population", "144,000,000", 144000000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_rw",
        name: "Rwanda",
        icon_name: "flag:rw",
        values: &[
            ("area_km2", "26,338", 26338.0),
            ("population", "14,000,000", 14000000.0),
            ("founded", "1962", 1962.0),
        ],
    },
    DefaultItem {
        id: "c_kn",
        name: "Saint Kitts and Nevis",
        icon_name: "flag:kn",
        values: &[
            ("area_km2", "261", 261.0),
            ("population", "47,000", 47000.0),
            ("founded", "1983", 1983.0),
        ],
    },
    DefaultItem {
        id: "c_lc",
        name: "Saint Lucia",
        icon_name: "flag:lc",
        values: &[
            ("area_km2", "616", 616.0),
            ("population", "180,000", 180000.0),
            ("founded", "1979", 1979.0),
        ],
    },
    DefaultItem {
        id: "c_vc",
        name: "Saint Vincent and the Grenadines",
        icon_name: "flag:vc",
        values: &[
            ("area_km2", "389", 389.0),
            ("population", "104,000", 104000.0),
            ("founded", "1979", 1979.0),
        ],
    },
    DefaultItem {
        id: "c_ws",
        name: "Samoa",
        icon_name: "flag:ws",
        values: &[
            ("area_km2", "2,842", 2842.0),
            ("population", "220,000", 220000.0),
            ("founded", "1962", 1962.0),
        ],
    },
    DefaultItem {
        id: "c_sm",
        name: "San Marino",
        icon_name: "flag:sm",
        values: &[
            ("area_km2", "61", 61.0),
            ("population", "34,000", 34000.0),
            ("founded", "301", 301.0),
        ],
    },
    DefaultItem {
        id: "c_st",
        name: "Sao Tome and Principe",
        icon_name: "flag:st",
        values: &[
            ("area_km2", "964", 964.0),
            ("population", "230,000", 230000.0),
            ("founded", "1975", 1975.0),
        ],
    },
    DefaultItem {
        id: "c_sa",
        name: "Saudi Arabia",
        icon_name: "flag:sa",
        values: &[
            ("area_km2", "2,149,690", 2149690.0),
            ("population", "36,000,000", 36000000.0),
            ("founded", "1932", 1932.0),
        ],
    },
    DefaultItem {
        id: "c_sn",
        name: "Senegal",
        icon_name: "flag:sn",
        values: &[
            ("area_km2", "196,722", 196722.0),
            ("population", "18,000,000", 18000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_rs",
        name: "Serbia",
        icon_name: "flag:rs",
        values: &[
            ("area_km2", "88,361", 88361.0),
            ("population", "6,600,000", 6600000.0),
            ("founded", "2006", 2006.0),
        ],
    },
    DefaultItem {
        id: "c_sc",
        name: "Seychelles",
        icon_name: "flag:sc",
        values: &[
            ("area_km2", "452", 452.0),
            ("population", "100,000", 100000.0),
            ("founded", "1976", 1976.0),
        ],
    },
    DefaultItem {
        id: "c_sl",
        name: "Sierra Leone",
        icon_name: "flag:sl",
        values: &[
            ("area_km2", "71,740", 71740.0),
            ("population", "8,600,000", 8600000.0),
            ("founded", "1961", 1961.0),
        ],
    },
    DefaultItem {
        id: "c_sg",
        name: "Singapore",
        icon_name: "flag:sg",
        values: &[
            ("area_km2", "728", 728.0),
            ("population", "5,900,000", 5900000.0),
            ("founded", "1965", 1965.0),
        ],
    },
    DefaultItem {
        id: "c_sk",
        name: "Slovakia",
        icon_name: "flag:sk",
        values: &[
            ("area_km2", "49,035", 49035.0),
            ("population", "5,400,000", 5400000.0),
            ("founded", "1993", 1993.0),
        ],
    },
    DefaultItem {
        id: "c_si",
        name: "Slovenia",
        icon_name: "flag:si",
        values: &[
            ("area_km2", "20,273", 20273.0),
            ("population", "2,100,000", 2100000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_sb",
        name: "Solomon Islands",
        icon_name: "flag:sb",
        values: &[
            ("area_km2", "28,896", 28896.0),
            ("population", "740,000", 740000.0),
            ("founded", "1978", 1978.0),
        ],
    },
    DefaultItem {
        id: "c_so",
        name: "Somalia",
        icon_name: "flag:so",
        values: &[
            ("area_km2", "637,657", 637657.0),
            ("population", "18,000,000", 18000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_za",
        name: "South Africa",
        icon_name: "flag:za",
        values: &[
            ("area_km2", "1,221,037", 1221037.0),
            ("population", "60,000,000", 60000000.0),
            ("founded", "1910", 1910.0),
        ],
    },
    DefaultItem {
        id: "c_kr",
        name: "South Korea",
        icon_name: "flag:kr",
        values: &[
            ("area_km2", "100,210", 100210.0),
            ("population", "51,700,000", 51700000.0),
            ("founded", "1948", 1948.0),
        ],
    },
    DefaultItem {
        id: "c_ss",
        name: "South Sudan",
        icon_name: "flag:ss",
        values: &[
            ("area_km2", "644,329", 644329.0),
            ("population", "11,000,000", 11000000.0),
            ("founded", "2011", 2011.0),
        ],
    },
    DefaultItem {
        id: "c_es",
        name: "Spain",
        icon_name: "flag:es",
        values: &[
            ("area_km2", "505,990", 505990.0),
            ("population", "48,000,000", 48000000.0),
            ("founded", "1479", 1479.0),
        ],
    },
    DefaultItem {
        id: "c_lk",
        name: "Sri Lanka",
        icon_name: "flag:lk",
        values: &[
            ("area_km2", "65,610", 65610.0),
            ("population", "22,000,000", 22000000.0),
            ("founded", "1948", 1948.0),
        ],
    },
    DefaultItem {
        id: "c_sd",
        name: "Sudan",
        icon_name: "flag:sd",
        values: &[
            ("area_km2", "1,861,484", 1861484.0),
            ("population", "48,000,000", 48000000.0),
            ("founded", "1956", 1956.0),
        ],
    },
    DefaultItem {
        id: "c_sr",
        name: "Suriname",
        icon_name: "flag:sr",
        values: &[
            ("area_km2", "163,820", 163820.0),
            ("population", "620,000", 620000.0),
            ("founded", "1975", 1975.0),
        ],
    },
    DefaultItem {
        id: "c_se",
        name: "Sweden",
        icon_name: "flag:se",
        values: &[
            ("area_km2", "450,295", 450295.0),
            ("population", "10,500,000", 10500000.0),
            ("founded", "1523", 1523.0),
        ],
    },
    DefaultItem {
        id: "c_ch",
        name: "Switzerland",
        icon_name: "flag:ch",
        values: &[
            ("area_km2", "41,285", 41285.0),
            ("population", "8,800,000", 8800000.0),
            ("founded", "1291", 1291.0),
        ],
    },
    DefaultItem {
        id: "c_sy",
        name: "Syria",
        icon_name: "flag:sy",
        values: &[
            ("area_km2", "185,180", 185180.0),
            ("population", "22,000,000", 22000000.0),
            ("founded", "1946", 1946.0),
        ],
    },
    DefaultItem {
        id: "c_tj",
        name: "Tajikistan",
        icon_name: "flag:tj",
        values: &[
            ("area_km2", "143,100", 143100.0),
            ("population", "10,000,000", 10000000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_tz",
        name: "Tanzania",
        icon_name: "flag:tz",
        values: &[
            ("area_km2", "947,303", 947303.0),
            ("population", "67,000,000", 67000000.0),
            ("founded", "1961", 1961.0),
        ],
    },
    DefaultItem {
        id: "c_th",
        name: "Thailand",
        icon_name: "flag:th",
        values: &[
            ("area_km2", "513,120", 513120.0),
            ("population", "71,000,000", 71000000.0),
            ("founded", "1238", 1238.0),
        ],
    },
    DefaultItem {
        id: "c_tl",
        name: "Timor-Leste",
        icon_name: "flag:tl",
        values: &[
            ("area_km2", "14,874", 14874.0),
            ("population", "1,400,000", 1400000.0),
            ("founded", "2002", 2002.0),
        ],
    },
    DefaultItem {
        id: "c_tg",
        name: "Togo",
        icon_name: "flag:tg",
        values: &[
            ("area_km2", "56,785", 56785.0),
            ("population", "9,000,000", 9000000.0),
            ("founded", "1960", 1960.0),
        ],
    },
    DefaultItem {
        id: "c_to",
        name: "Tonga",
        icon_name: "flag:to",
        values: &[
            ("area_km2", "747", 747.0),
            ("population", "100,000", 100000.0),
            ("founded", "1970", 1970.0),
        ],
    },
    DefaultItem {
        id: "c_tt",
        name: "Trinidad and Tobago",
        icon_name: "flag:tt",
        values: &[
            ("area_km2", "5,130", 5130.0),
            ("population", "1,500,000", 1500000.0),
            ("founded", "1962", 1962.0),
        ],
    },
    DefaultItem {
        id: "c_tn",
        name: "Tunisia",
        icon_name: "flag:tn",
        values: &[
            ("area_km2", "163,610", 163610.0),
            ("population", "12,000,000", 12000000.0),
            ("founded", "1956", 1956.0),
        ],
    },
    DefaultItem {
        id: "c_tr",
        name: "Turkey",
        icon_name: "flag:tr",
        values: &[
            ("area_km2", "783,562", 783562.0),
            ("population", "85,000,000", 85000000.0),
            ("founded", "1923", 1923.0),
        ],
    },
    DefaultItem {
        id: "c_tm",
        name: "Turkmenistan",
        icon_name: "flag:tm",
        values: &[
            ("area_km2", "488,100", 488100.0),
            ("population", "6,500,000", 6500000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_tv",
        name: "Tuvalu",
        icon_name: "flag:tv",
        values: &[
            ("area_km2", "26", 26.0),
            ("population", "11,000", 11000.0),
            ("founded", "1978", 1978.0),
        ],
    },
    DefaultItem {
        id: "c_ug",
        name: "Uganda",
        icon_name: "flag:ug",
        values: &[
            ("area_km2", "241,550", 241550.0),
            ("population", "48,000,000", 48000000.0),
            ("founded", "1962", 1962.0),
        ],
    },
    DefaultItem {
        id: "c_ua",
        name: "Ukraine",
        icon_name: "flag:ua",
        values: &[
            ("area_km2", "603,500", 603500.0),
            ("population", "38,000,000", 38000000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_ae",
        name: "United Arab Emirates",
        icon_name: "flag:ae",
        values: &[
            ("area_km2", "83,600", 83600.0),
            ("population", "9,500,000", 9500000.0),
            ("founded", "1971", 1971.0),
        ],
    },
    DefaultItem {
        id: "c_gb",
        name: "United Kingdom",
        icon_name: "flag:gb",
        values: &[
            ("area_km2", "242,495", 242495.0),
            ("population", "67,000,000", 67000000.0),
            ("founded", "1801", 1801.0),
        ],
    },
    DefaultItem {
        id: "c_us",
        name: "United States",
        icon_name: "flag:us",
        values: &[
            ("area_km2", "9,833,517", 9833517.0),
            ("population", "335,000,000", 335000000.0),
            ("founded", "1776", 1776.0),
        ],
    },
    DefaultItem {
        id: "c_uy",
        name: "Uruguay",
        icon_name: "flag:uy",
        values: &[
            ("area_km2", "181,034", 181034.0),
            ("population", "3,400,000", 3400000.0),
            ("founded", "1825", 1825.0),
        ],
    },
    DefaultItem {
        id: "c_uz",
        name: "Uzbekistan",
        icon_name: "flag:uz",
        values: &[
            ("area_km2", "447,400", 447400.0),
            ("population", "36,000,000", 36000000.0),
            ("founded", "1991", 1991.0),
        ],
    },
    DefaultItem {
        id: "c_vu",
        name: "Vanuatu",
        icon_name: "flag:vu",
        values: &[
            ("area_km2", "12,189", 12189.0),
            ("population", "330,000", 330000.0),
            ("founded", "1980", 1980.0),
        ],
    },
    DefaultItem {
        id: "c_va",
        name: "Vatican City",
        icon_name: "flag:va",
        values: &[
            ("area_km2", "0.49", 0.49),
            ("population", "800", 800.0),
            ("founded", "1929", 1929.0),
        ],
    },
    DefaultItem {
        id: "c_ve",
        name: "Venezuela",
        icon_name: "flag:ve",
        values: &[
            ("area_km2", "916,445", 916445.0),
            ("population", "28,000,000", 28000000.0),
            ("founded", "1811", 1811.0),
        ],
    },
    DefaultItem {
        id: "c_vn",
        name: "Vietnam",
        icon_name: "flag:vn",
        values: &[
            ("area_km2", "331,212", 331212.0),
            ("population", "99,000,000", 99000000.0),
            ("founded", "1945", 1945.0),
        ],
    },
    DefaultItem {
        id: "c_ye",
        name: "Yemen",
        icon_name: "flag:ye",
        values: &[
            ("area_km2", "527,968", 527968.0),
            ("population", "34,000,000", 34000000.0),
            ("founded", "1990", 1990.0),
        ],
    },
    DefaultItem {
        id: "c_zm",
        name: "Zambia",
        icon_name: "flag:zm",
        values: &[
            ("area_km2", "752,612", 752612.0),
            ("population", "20,000,000", 20000000.0),
            ("founded", "1964", 1964.0),
        ],
    },
    DefaultItem {
        id: "c_zw",
        name: "Zimbabwe",
        icon_name: "flag:zw",
        values: &[
            ("area_km2", "390,757", 390757.0),
            ("population", "16,000,000", 16000000.0),
            ("founded", "1980", 1980.0),
        ],
    },
];

/// The four lists the draft ships with, per the spec's Assumptions.
pub const DEFAULT_LISTS: [DefaultList; 4] = [
    DefaultList {
        id: "default_world_history",
        title: "World History & Inventions",
        description: "Key human milestones from the printing press to the smartphone",
        order_direction: "low_to_high",
        columns: HISTORY_COLUMNS,
        items: HISTORY_ITEMS,
    },
    DefaultList {
        id: "default_statues_monuments",
        title: "Iconic Statues & Architectural Marvels",
        description: "World wonders with height and construction date columns",
        order_direction: "low_to_high",
        columns: STATUE_COLUMNS,
        items: STATUE_ITEMS,
    },
    DefaultList {
        id: "default_cinema_box_office",
        title: "Blockbuster Cinema Box Office",
        description: "Record-setting movies with box office revenue and budget",
        order_direction: "low_to_high",
        columns: CINEMA_COLUMNS,
        items: CINEMA_ITEMS,
    },
    DefaultList {
        id: "default_countries",
        title: "Countries of the World",
        description: "Every sovereign state with its flag, area, population, and founding year",
        order_direction: "low_to_high",
        columns: COUNTRIES_COLUMNS,
        items: COUNTRIES_ITEMS,
    },
];

/// One column as the API reports it.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnSummary {
    pub id: String,
    pub name: String,
}

/// A list as the API reports it: enough to pick a round, never the items.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub order_direction: String,
    pub columns: Vec<ColumnSummary>,
    pub item_count: i64,
    pub is_default: bool,
}

/// The `{ "lists": [...] }` envelope the list endpoints return.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ListSummaries {
    pub lists: Vec<ListSummary>,
}

/// Every list in `folder_id` with its columns, oldest first. Reads the database
/// rather than the seed constants, so a folder created later answers the same
/// way the seeded one does.
pub fn list_summaries(conn: &Connection, folder_id: &str) -> ApiResult<ListSummaries> {
    let mut lists = Vec::new();
    let mut statement = conn.prepare(
        "SELECT id, title, description, order_direction, item_count, is_default
         FROM quiz_list WHERE folder_id = ?1 ORDER BY created_at ASC, title ASC",
    )?;
    let rows = statement.query_map(params![folder_id], |row| {
        Ok(ListSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            order_direction: row.get(3)?,
            columns: Vec::new(),
            item_count: row.get(4)?,
            is_default: row.get::<_, i64>(5)? != 0,
        })
    })?;
    for row in rows {
        let mut list = row?;
        list.columns = columns_of(conn, &list.id)?;
        lists.push(list);
    }
    Ok(ListSummaries { lists })
}

/// A list's columns in display order; `columns[0]` is the default value column.
fn columns_of(conn: &Connection, list_id: &str) -> ApiResult<Vec<ColumnSummary>> {
    let mut statement = conn.prepare(
        "SELECT id, name FROM list_item_column WHERE list_id = ?1 ORDER BY position ASC",
    )?;
    let rows = statement.query_map(params![list_id], |row| {
        Ok(ColumnSummary {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    let columns = rows.collect::<Result<Vec<_>, _>>()?;
    Ok(columns)
}

/// Insert the seeded lists into `folder_id`. Idempotent, and a failure to write the
/// seed is reported rather than swallowed.
pub fn seed_default_lists(conn: &Connection, folder_id: &str) -> ApiResult<()> {
    let tx = conn.unchecked_transaction()?;
    for list in DEFAULT_LISTS.iter() {
        tx.execute(
            "INSERT INTO quiz_list
                (id, folder_id, title, description, order_direction, item_count, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?7)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                order_direction = excluded.order_direction,
                item_count = excluded.item_count",
            params![
                list.id,
                folder_id,
                list.title,
                list.description,
                list.order_direction,
                list.items.len() as i64,
                SEED_TIME,
            ],
        )?;
        for (position, column) in list.columns.iter().enumerate() {
            tx.execute(
                "INSERT OR REPLACE INTO list_item_column (list_id, id, name, position)
                 VALUES (?1, ?2, ?3, ?4)",
                params![list.id, column.id, column.name, position as i64],
            )?;
        }
        for (position, item) in list.items.iter().enumerate() {
            tx.execute(
                "INSERT OR IGNORE INTO item (id, list_id, name, position, icon_name)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![item.id, list.id, item.name, position as i64, item.icon_name],
            )?;
            for (column_id, display, numeric) in item.values.iter() {
                tx.execute(
                    "INSERT OR REPLACE INTO item_value (item_id, column_id, display, numeric, was_defaulted)
                     VALUES (?1, ?2, ?3, ?4, 0)",
                    params![item.id, column_id, display, numeric],
                )?;
            }
        }
    }
    tx.commit()?;
    Ok(())
}

/// Read a list's items ordered by position.
pub fn list_item_count(conn: &Connection, list_id: &str) -> ApiResult<usize> {
    let count = conn
        .query_row(
            "SELECT COUNT(*) FROM item WHERE list_id = ?1",
            params![list_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .unwrap_or(0);
    Ok(count as usize)
}

/// An Order round needs one anchor plus ten candidates, hence the minimum of 11
/// (spec Assumptions).
pub fn validate_list_size(count: usize) -> ApiResult<()> {
    if count < MIN_ITEMS {
        return Err(ApiError::new(
            crate::errors::ErrorCode::TooSmall,
            format!("A list needs at least {MIN_ITEMS} items; this one has {count}."),
        ));
    }
    if count > MAX_ITEMS {
        return Err(ApiError::new(
            crate::errors::ErrorCode::TooLarge,
            format!("A list cannot hold more than {MAX_ITEMS} items; this one has {count}."),
        ));
    }
    Ok(())
}

pub fn validate_list_title(title: &str) -> ApiResult<()> {
    let trimmed = title.trim();
    let len = trimmed.chars().count();
    if len == 0 {
        return Err(ApiError::bad_request("List title cannot be empty."));
    }
    if len > MAX_LIST_TITLE_CHARS {
        return Err(ApiError::bad_request(format!(
            "List title cannot exceed {MAX_LIST_TITLE_CHARS} characters."
        )));
    }
    Ok(())
}

pub fn validate_column_name(name: &str) -> ApiResult<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request("Column name cannot be empty."));
    }
    if trimmed.chars().count() > MAX_COLUMN_NAME_CHARS {
        return Err(ApiError::bad_request(format!(
            "Column name cannot exceed {MAX_COLUMN_NAME_CHARS} characters."
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
