use regex::Regex;

pub struct NavRenderer {
    re: Regex,
}

impl NavRenderer {
    pub fn init() -> NavRenderer {
        NavRenderer {
            re: Regex::new("<h(\\d)>.*?id=\"([a-z0-9\\-]+)\"></a>(.*?)</h\\d>").unwrap(),
        }
    }

    pub fn render(&self, content: &str) -> String {
        let mut res = String::new();
        let mut last_header = 0;
        let mut list_level = 1;
        res += "<ul>";
        for cap in self.re.captures_iter(content) {
            let lvl: usize = cap[1].parse().unwrap();
            let fmt = format!(
                "<a class=\"h{}\" href=\"#{}\">{}</a>",
                &cap[1], &cap[2], &cap[3],
            );

            if lvl > last_header {
                while lvl > list_level {
                    res += "<li><ul>";
                    list_level += 1;
                }
                res += "<li>";
                res += fmt.as_str();
                res += "</li>\n";
                list_level = lvl;
            } else {
                while list_level > lvl {
                    res += "</ul></li>\n";
                    list_level -= 1;
                }
                res += "<li>";
                res += fmt.as_str();
                res += "</li>";
                res += "\n";
            }
            last_header = lvl;
        }
        while list_level > 1 {
            res += "</ul></li>\n";
            list_level -= 1;
        }
        res += "</ul>";
        res
    }
}
