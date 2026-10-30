use std::collections::BTreeMap;

#[derive(Debug, Clone)]
enum Json {
    Null, Bool(bool), Number(f64), String(String), Array(Vec<Json>), Object(BTreeMap<String, Json>),
}

struct Parser { chars: Vec<char>, pos: usize }
impl Parser {
    fn new(s: &str) -> Self { Self { chars: s.chars().collect(), pos: 0 } }
    fn ws(&mut self) { while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() { self.pos += 1; } }
    fn peek(&mut self) -> Option<char> { self.ws(); self.chars.get(self.pos).copied() }
    fn take(&mut self) -> Option<char> { let c = self.chars.get(self.pos).copied(); if c.is_some() { self.pos += 1; } c }
    fn expect(&mut self, wanted: char) -> Result<(), String> {
        if self.take() == Some(wanted) { Ok(()) } else { Err(format!("expected '{wanted}' in JSON at character {}", self.pos)) }
    }
    fn string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.take().ok_or("unterminated JSON string")? {
                '"' => return Ok(out),
                '\\' => match self.take().ok_or("unterminated JSON escape")? {
                    '"' => out.push('"'), '\\' => out.push('\\'), '/' => out.push('/'),
                    'b' => out.push('\u{0008}'), 'f' => out.push('\u{000c}'), 'n' => out.push('\n'),
                    'r' => out.push('\r'), 't' => out.push('\t'),
                    'u' => {
                        let first = self.hex4()?;
                        let code = if (0xd800..=0xdbff).contains(&first) {
                            if self.take() != Some('\\') || self.take() != Some('u') { return Err("invalid JSON surrogate pair".into()); }
                            let second = self.hex4()?;
                            if !(0xdc00..=0xdfff).contains(&second) { return Err("invalid JSON surrogate pair".into()); }
                            0x10000 + (((first - 0xd800) as u32) << 10) + (second - 0xdc00) as u32
                        } else if (0xdc00..=0xdfff).contains(&first) { return Err("unpaired JSON low surrogate".into()); }
                        else { first as u32 };
                        out.push(char::from_u32(code).ok_or("invalid JSON unicode escape")?);
                    }
                    c => return Err(format!("invalid JSON escape '\\{c}'")),
                },
                c if c < '\u{20}' => return Err("unescaped control character in JSON string".into()),
                c => out.push(c),
            }
        }
    }
    fn hex4(&mut self) -> Result<u16, String> {
        let mut n = 0u16;
        for _ in 0..4 {
            let c = self.take().ok_or("short JSON unicode escape")?;
            n = n.checked_mul(16).ok_or("invalid JSON unicode escape")?
                + c.to_digit(16).ok_or("invalid JSON unicode escape")? as u16;
        }
        Ok(n)
    }
    fn value(&mut self) -> Result<Json, String> {
        match self.peek().ok_or("unexpected end of JSON")? {
            '"' => Ok(Json::String(self.string()?)),
            '[' => {
                self.take(); let mut a = Vec::new();
                if self.peek() == Some(']') { self.take(); return Ok(Json::Array(a)); }
                loop { a.push(self.value()?); match self.peek() { Some(',') => { self.take(); }, Some(']') => { self.take(); break; }, _ => return Err("expected ',' or ']' in JSON array".into()) } }
                Ok(Json::Array(a))
            }
            '{' => {
                self.take(); let mut o = BTreeMap::new();
                if self.peek() == Some('}') { self.take(); return Ok(Json::Object(o)); }
                loop {
                    if self.peek() != Some('"') { return Err("JSON object keys must be strings".into()); }
                    let k = self.string()?; self.expect(':')?; let v = self.value()?;
                    if o.insert(k.clone(), v).is_some() { return Err(format!("duplicate JSON object key '{k}'")); }
                    match self.peek() { Some(',') => { self.take(); }, Some('}') => { self.take(); break; }, _ => return Err("expected ',' or '}' in JSON object".into()) }
                }
                Ok(Json::Object(o))
            }
            't' => { self.literal("true")?; Ok(Json::Bool(true)) }
            'f' => { self.literal("false")?; Ok(Json::Bool(false)) }
            'n' => { self.literal("null")?; Ok(Json::Null) }
            '-' | '0'..='9' => self.number(),
            c => Err(format!("unexpected character '{c}' in JSON")),
        }
    }
    fn literal(&mut self, s: &str) -> Result<(), String> { for c in s.chars() { if self.take() != Some(c) { return Err("invalid JSON literal".into()); } } Ok(()) }
    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        if self.peek() == Some('-') { self.take(); }
        match self.peek() {
            Some('0') => { self.take(); if matches!(self.peek(), Some('0'..='9')) { return Err("leading zero in JSON number".into()); } }
            Some('1'..='9') => { self.take(); while matches!(self.peek(), Some('0'..='9')) { self.take(); } }
            _ => return Err("invalid JSON number".into()),
        }
        if self.peek() == Some('.') { self.take(); if !matches!(self.peek(), Some('0'..='9')) { return Err("invalid JSON fraction".into()); } while matches!(self.peek(), Some('0'..='9')) { self.take(); } }
        if matches!(self.peek(), Some('e'|'E')) {
            self.take(); if matches!(self.peek(), Some('+'|'-')) { self.take(); }
            if !matches!(self.peek(), Some('0'..='9')) { return Err("invalid JSON exponent".into()); }
            while matches!(self.peek(), Some('0'..='9')) { self.take(); }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        let n: f64 = s.trim().parse().map_err(|_| "invalid JSON number")?;
        if !n.is_finite() { return Err("JSON numbers must be finite".into()); }
        Ok(Json::Number(n))
    }
}
fn parse_json(s: &str) -> Result<Json, String> {
    let mut p = Parser::new(s); let v = p.value()?; p.ws();
    if p.pos != p.chars.len() { return Err("trailing content after JSON document".into()); }
    Ok(v)
}
fn object<'a>(v: &'a Json, name: &str) -> Result<&'a BTreeMap<String, Json>, String> {
    if let Json::Object(o) = v { Ok(o) } else { Err(format!("{name} must be an object")) }
}
fn array<'a>(v: &'a Json, name: &str) -> Result<&'a [Json], String> {
    if let Json::Array(a) = v { Ok(a) } else { Err(format!("{name} must be an array")) }
}
fn string<'a>(v: &'a Json, name: &str) -> Result<&'a str, String> {
    if let Json::String(s) = v { Ok(s) } else { Err(format!("{name} must be a string")) }
}
fn number(v: &Json, name: &str) -> Result<f64, String> {
    if let Json::Number(n) = v { Ok(*n) } else { Err(format!("{name} must be a number")) }
}
fn boolean(v: &Json, name: &str) -> Result<bool, String> {
    if let Json::Bool(b) = v { Ok(*b) } else { Err(format!("{name} must be a boolean")) }
}
fn required<'a>(o: &'a BTreeMap<String, Json>, key: &str, parent: &str) -> Result<&'a Json, String> {
    o.get(key).ok_or_else(|| format!("{parent}.{key} is required"))
}
fn keys(o: &BTreeMap<String, Json>, allowed: &[&str], parent: &str) -> Result<(), String> {
    for k in o.keys() { if !allowed.contains(&k.as_str()) { return Err(format!("unknown field '{parent}.{k}'")); } }
    Ok(())
}
fn color(v: &Json, name: &str) -> Result<String, String> {
    let s = string(v, name)?;
    if s.len() != 7 || !s.starts_with('#') || !s[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("{name} must be a six-digit hexadecimal color"));
    }
    Ok(s.to_string())
}

fn scene_get<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> &'a Json {
    object.get(key).expect("required scene property")
}

fn scene_num(value: &Json) -> f64 {
    match value { Json::Number(number) => *number, _ => 0.0 }
}

fn scene_str(value: &Json) -> &str {
    match value { Json::String(string) => string.as_str(), _ => "" }
}

fn scene_bool(value: &Json) -> bool {
    matches!(value, Json::Bool(true))
}


fn aoe_order(values: &[Vec<Option<f64>>]) -> Result<Vec<usize>, String> {
    let n = values.len();
    if n < 2 { return Err("plot.ordering 'aoe' requires at least two variables".into()); }
    let mut matrix = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            matrix[i][j] = values[i][j].ok_or_else(|| {
                format!("plot.ordering 'aoe' requires complete correlations; missing value at ({}, {})", i + 1, j + 1)
            })?;
        }
    }
    let mut vectors = vec![vec![0.0; n]; n];
    for i in 0..n { vectors[i][i] = 1.0; }
    let tolerance = 1e-12;
    let mut converged = false;
    for _ in 0..(100 * n * n) {
        let mut p = 0;
        let mut q = 1;
        let mut largest = 0.0_f64;
        for i in 0..n {
            for j in (i + 1)..n {
                let candidate = matrix[i][j].abs();
                if candidate > largest { largest = candidate; p = i; q = j; }
            }
        }
        if largest <= tolerance { converged = true; break; }
        let app = matrix[p][p];
        let aqq = matrix[q][q];
        let apq = matrix[p][q];
        let tau = (aqq - app) / (2.0 * apq);
        let sign = if tau >= 0.0 { 1.0 } else { -1.0 };
        let t = if tau == 0.0 { 1.0 } else { sign / (tau.abs() + (1.0 + tau * tau).sqrt()) };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;
        for k in 0..n {
            if k != p && k != q {
                let akp = matrix[k][p];
                let akq = matrix[k][q];
                matrix[k][p] = c * akp - s * akq;
                matrix[p][k] = matrix[k][p];
                matrix[k][q] = s * akp + c * akq;
                matrix[q][k] = matrix[k][q];
            }
            let vkp = vectors[k][p];
            let vkq = vectors[k][q];
            vectors[k][p] = c * vkp - s * vkq;
            vectors[k][q] = s * vkp + c * vkq;
        }
        matrix[p][p] = app - t * apq;
        matrix[q][q] = aqq + t * apq;
        matrix[p][q] = 0.0;
        matrix[q][p] = 0.0;
    }
    if !converged { return Err("plot.ordering 'aoe' eigen decomposition did not converge".into()); }
    let mut eigen_order: Vec<usize> = (0..n).collect();
    eigen_order.sort_by(|a, b| matrix[*b][*b].total_cmp(&matrix[*a][*a]).then(a.cmp(b)));
    let first = eigen_order[0];
    let second = eigen_order[1];
    let scale = matrix[first][first].abs().max(matrix[second][second].abs()).max(1.0);
    if (matrix[first][first] - matrix[second][second]).abs() <= 1e-10 * scale {
        return Err("plot.ordering 'aoe' is undefined when the two largest eigenvalues are tied".into());
    }
    for column in [first, second] {
        let mut pivot = 0;
        for row in 1..n {
            if vectors[row][column].abs() > vectors[pivot][column].abs() { pivot = row; }
        }
        if vectors[pivot][column] < 0.0 {
            for row in 0..n { vectors[row][column] = -vectors[row][column]; }
        }
    }
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|a, b| {
        let angle_a = vectors[*a][second].atan2(vectors[*a][first]);
        let angle_b = vectors[*b][second].atan2(vectors[*b][first]);
        angle_a.total_cmp(&angle_b).then(a.cmp(b))
    });
    Ok(indices)
}

fn hclust_order(values: &[Vec<Option<f64>>]) -> Result<Vec<usize>, String> {
    let n = values.len();
    if n < 2 { return Err("plot.ordering 'hclust' requires at least two variables".into()); }
    let mut distances = vec![vec![None; n]; n];
    for i in 0..n {
        let mut observed = 0;
        for j in 0..n {
            if i == j { continue; }
            if let Some(correlation) = values[i][j] {
                distances[i][j] = Some(1.0 - correlation);
                observed += 1;
            }
        }
        if observed == 0 {
            return Err(format!("plot.ordering 'hclust' cannot cluster '{}' because it has no observed off-diagonal correlation", i + 1));
        }
    }
    let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    while clusters.len() > 1 {
        let mut best: Option<(f64, usize, usize, usize, usize)> = None;
        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let mut sum = 0.0;
                let mut count = 0usize;
                for &a in &clusters[i] {
                    for &b in &clusters[j] {
                        if let Some(distance) = distances[a][b] { sum += distance; count += 1; }
                    }
                }
                if count == 0 { continue; }
                let distance = sum / count as f64;
                let key_i = *clusters[i].iter().min().unwrap();
                let key_j = *clusters[j].iter().min().unwrap();
                let (low, high) = if key_i < key_j { (key_i, key_j) } else { (key_j, key_i) };
                let replace = match best {
                    None => true,
                    Some(current) => distance.total_cmp(&current.0)
                        .then(low.cmp(&current.1))
                        .then(high.cmp(&current.2)).is_lt(),
                };
                if replace { best = Some((distance, low, high, i, j)); }
            }
        }
        let (_, _, _, i, j) = best.ok_or(
            "plot.ordering 'hclust' cannot merge remaining clusters: no observed cross-cluster correlations"
        )?;
        let right = clusters.remove(j);
        let left = clusters.remove(i);
        let (mut merged, other) = if left.iter().min() < right.iter().min() {
            (left, right)
        } else {
            (right, left)
        };
        merged.extend(other);
        clusters.push(merged);
    }
    Ok(clusters.pop().unwrap())
}

fn variable_order(
    ordering: &str,
    kind: &str,
    labels: &[String],
    values: &[Vec<Option<f64>>],
) -> Result<Vec<usize>, String> {
    match ordering {
        "original" => Ok((0..labels.len()).collect()),
        "alphabetical" => {
            let mut indices: Vec<usize> = (0..labels.len()).collect();
            indices.sort_by(|a, b| labels[*a].to_lowercase().cmp(&labels[*b].to_lowercase())
                .then(labels[*a].cmp(&labels[*b])).then(a.cmp(b)));
            Ok(indices)
        }
        "aoe" => {
            if kind != "correlation" { return Err("plot.ordering 'aoe' is supported only for correlation matrices".into()); }
            aoe_order(values)
        }
        "hclust" => {
            if kind != "correlation" { return Err("plot.ordering 'hclust' is supported only for correlation matrices".into()); }
            hclust_order(values)
        }
        _ => Err(format!("plot.ordering '{ordering}' is not supported by the native CLI")),
    }
}

#[derive(Debug, Clone)]
pub struct Publication {
    pub labels: Vec<String>, pub values: Vec<Vec<Option<f64>>>,
    pub original_labels: Vec<String>, pub order_indices: Vec<usize>,
    pub kind: String, pub method: String, pub display: String, pub ordering: String,
    pub show_values: bool, pub negative: String, pub neutral: String, pub positive: String,
    pub missing: String, pub width: f64, pub height: f64, pub unit: String,
    pub family: String, pub base_size: f64, pub output: String,
}
impl Publication {
    pub fn load(config_path: &str) -> Result<Self, String> {
        let source = std::fs::read_to_string(config_path).map_err(|e| format!("cannot read config '{config_path}': {e}"))?;
        let root = parse_json(&source)?;
        let root = object(&root, "config")?;
        keys(root, &["config_version","input","matrix","plot","colors","dimensions","typography","output"], "config")?;
        if number(required(root,"config_version","config")?, "config.config_version")? != 1.0 { return Err("config.config_version must equal 1".into()); }

        let matrix_cfg = object(required(root,"matrix","config")?, "matrix")?;
        keys(matrix_cfg, &["kind","method","missing","col_limits"], "matrix")?;
        let kind = string(required(matrix_cfg,"kind","matrix")?, "matrix.kind")?.to_string();
        if kind != "correlation" && kind != "general_numeric" { return Err("matrix.kind must be correlation or general_numeric".into()); }
        if matrix_cfg.contains_key("col_limits") { return Err("matrix.col_limits is not yet supported by the native CLI".into()); }
        if kind == "correlation" {
            let method = string(required(matrix_cfg,"method","matrix")?, "matrix.method")?;
            if !["pearson","spearman","kendall"].contains(&method) { return Err("unsupported matrix.method".into()); }
        } else if matrix_cfg.contains_key("method") { return Err("matrix.method is only valid for correlation matrices".into()); }
        let missing = string(required(matrix_cfg,"missing","matrix")?, "matrix.missing")?;
        if missing != "preserve" && missing != "error" { return Err("matrix.missing must be preserve or error".into()); }

        let input = object(required(root,"input","config")?, "input")?;
        let input_kind = string(required(input,"kind","input")?, "input.kind")?;
        let (mut labels, mut values) = match input_kind {
            "inline_matrix" => {
                keys(input, &["kind","labels","values"], "input")?;
                let labels: Vec<String> = array(required(input,"labels","input")?, "input.labels")?.iter().enumerate()
                    .map(|(i,v)| string(v,&format!("input.labels[{i}]")).map(str::to_string)).collect::<Result<_,_>>()?;
                let rows = array(required(input,"values","input")?, "input.values")?;
                let mut values = Vec::new();
                for (r,row) in rows.iter().enumerate() {
                    let mut out = Vec::new();
                    for (c,v) in array(row,&format!("input.values[{r}]"))?.iter().enumerate() {
                        out.push(match v { Json::Null => None, Json::Number(n) => Some(*n), _ => return Err(format!("input.values[{r}][{c}] must be a number or null")) });
                    }
                    values.push(out);
                }
                (labels, values)
            }
            "matrix_csv" => {
                keys(input, &["kind","path","delimiter","header","row_labels"], "input")?;
                let path = string(required(input,"path","input")?, "input.path")?;
                let delimiter = string(required(input,"delimiter","input")?, "input.delimiter")?;
                if delimiter != "," && delimiter != "\t" { return Err("native CLI currently supports only comma or tab delimiters".into()); }
                if !boolean(required(input,"header","input")?, "input.header")? || !boolean(required(input,"row_labels","input")?, "input.row_labels")? {
                    return Err("matrix_csv currently requires header=true and row_labels=true".into());
                }
                let base = std::path::Path::new(config_path).parent().unwrap_or_else(|| std::path::Path::new("."));
                let path = base.join(path);
                let text = std::fs::read_to_string(&path).map_err(|e| format!("cannot read matrix CSV '{}': {e}", path.display()))?;
                let parsed = crate::parse_matrix(&text, if kind == "correlation" { crate::Kind::Correlation } else { crate::Kind::General })?;
                (parsed.labels, parsed.values)
            }
            other => return Err(format!("unsupported input.kind '{other}'")),
        };
        if labels.is_empty() { return Err("input.labels must not be empty".into()); }
        let n = labels.len();
        let mut seen = std::collections::HashSet::new();
        if labels.iter().any(|s| s.trim().is_empty() || !seen.insert(s)) { return Err("input.labels must be non-empty and unique".into()); }
        if values.len() != n || values.iter().any(|r| r.len() != n) { return Err("input.values must be a square matrix matching labels".into()); }
        if values.iter().flatten().all(Option::is_none) { return Err("matrix contains no numeric values".into()); }
        if missing == "error" && values.iter().flatten().any(Option::is_none) { return Err("matrix.missing is 'error' but matrix contains missing values".into()); }
        for (r,row) in values.iter().enumerate() {
            for (c,v) in row.iter().enumerate() {
                if let Some(x) = v { if !x.is_finite() || (kind == "correlation" && !(-1.0..=1.0).contains(x)) {
                    return Err(format!("invalid matrix value at row {}, column '{}'", r+1, labels[c]));
                }}
            }
        }
        if kind == "correlation" {
            for r in 0..n { for c in 0..n {
                match (values[r][c], values[c][r]) {
                    (Some(a),Some(b)) if (a-b).abs() <= 1e-8 => {},
                    (None,None) => {},
                    _ => return Err(format!("correlation matrix is not symmetric at ('{}', '{}')",labels[r],labels[c])),
                }
            }}
            for i in 0..n { if values[i][i].map_or(true,|v|(v-1.0).abs()>1e-8) { return Err(format!("correlation diagonal for '{}' must equal 1",labels[i])); } }
        }

        let plot = object(required(root,"plot","config")?, "plot")?;
        keys(plot,&["method","display","ordering","show_values","significance"],"plot")?;
        let method = string(required(plot,"method","plot")?,"plot.method")?.to_string();
        if method != "circle" && method != "number" { return Err(format!("plot.method '{method}' is not yet supported by the native CLI")); }
        let display = string(required(plot,"display","plot")?,"plot.display")?.to_string();
        if !["full","upper","lower"].contains(&display.as_str()) { return Err(format!("plot.display '{display}' is not yet supported by the native CLI")); }
        let ordering = string(required(plot,"ordering","plot")?,"plot.ordering")?.to_string();
        if !["original", "alphabetical", "aoe", "hclust"].contains(&ordering.as_str()) {
            return Err(format!("plot.ordering '{ordering}' is not supported"));
        }
        let show_values = boolean(required(plot,"show_values","plot")?,"plot.show_values")?;
        let sig = object(required(plot,"significance","plot")?,"plot.significance")?;
        keys(sig,&["enabled","p_values","thresholds"],"plot.significance")?;
        if boolean(required(sig,"enabled","plot.significance")?,"plot.significance.enabled")? { return Err("plot.significance is not yet supported by the native CLI".into()); }
        let _ = required(sig,"p_values","plot.significance")?;
        let thresholds = array(required(sig,"thresholds","plot.significance")?,"plot.significance.thresholds")?;
        if thresholds.is_empty() { return Err("plot.significance.thresholds must not be empty".into()); }
        for v in thresholds { let x=number(v,"plot.significance.thresholds[]")?; if x<=0.0 || x>1.0 { return Err("significance thresholds must be in (0, 1]".into()); } }

        let colors = object(required(root,"colors","config")?,"colors")?;
        keys(colors,&["negative","neutral","positive","missing","scale"],"colors")?;
        if colors.contains_key("scale") { return Err("colors.scale is not yet supported by the native CLI".into()); }
        let negative=color(required(colors,"negative","colors")?,"colors.negative")?;
        let neutral=color(required(colors,"neutral","colors")?,"colors.neutral")?;
        let positive=color(required(colors,"positive","colors")?,"colors.positive")?;
        let missing_color=color(required(colors,"missing","colors")?,"colors.missing")?;

        let dims=object(required(root,"dimensions","config")?,"dimensions")?;
        keys(dims,&["width","height","unit"],"dimensions")?;
        let width=number(required(dims,"width","dimensions")?,"dimensions.width")?;
        let height=number(required(dims,"height","dimensions")?,"dimensions.height")?;
        let unit=string(required(dims,"unit","dimensions")?,"dimensions.unit")?.to_string();
        if width<=0.0 || height<=0.0 || !["mm","in"].contains(&unit.as_str()) { return Err("dimensions require positive width/height and unit mm or in".into()); }

        let typography=object(required(root,"typography","config")?,"typography")?;
        keys(typography,&["family","base_size"],"typography")?;
        let family=string(required(typography,"family","typography")?,"typography.family")?.to_string();
        let base_size=number(required(typography,"base_size","typography")?,"typography.base_size")?;
        if family.trim().is_empty() || base_size<=0.0 { return Err("typography.family and typography.base_size must be positive/non-empty".into()); }

        let output=object(required(root,"output","config")?,"output")?;
        keys(output,&["format","path"],"output")?;
        if string(required(output,"format","output")?,"output.format")? != "svg" { return Err("output.format must be svg".into()); }
        let output_path=string(required(output,"path","output")?,"output.path")?.to_string();
        if output_path.trim().is_empty() { return Err("output.path must not be empty".into()); }

        let original_labels = labels.clone();
        let order_indices = variable_order(&ordering, &kind, &labels, &values)?;
        if ordering != "original" {
            let old_labels = labels.clone();
            let old_values = values.clone();
            labels = order_indices.iter().map(|i| old_labels[*i].clone()).collect();
            values = order_indices.iter()
                .map(|i| order_indices.iter().map(|j| old_values[*i][*j]).collect())
                .collect();
        }
        Ok(Self { labels, values, original_labels, order_indices, kind, method, display, ordering, show_values, negative, neutral, positive,
            missing: missing_color, width, height, unit, family, base_size, output: output_path })
    }

    pub fn scene_json(&self, title: &str) -> String {
        let n = self.labels.len();
        let cell = 64.0;
        let left = 120.0;
        let top = 76.0;
        let width = left + n as f64 * cell + 24.0;
        let height = top + n as f64 * cell + 32.0;
        let esc = |value: &str| -> String {
            let mut out = String::new();
            for ch in value.chars() {
                match ch {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c < '\u{20}' => out.push_str(&format!("\\u{:04x}", c as u32)),
                    c => out.push(c),
                }
            }
            out
        };
        let fmt = |v: f64| if v == 0.0 { "0".to_string() } else { format!("{v}") };
        let (lower, upper, intercept, zoom, mapping) = if self.kind == "general_numeric" {
            let observed: Vec<f64> = self.values.iter().flatten().filter_map(|v| *v).collect();
            let mut lo = observed.iter().copied().fold(f64::INFINITY, f64::min);
            let mut hi = observed.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            if lo == hi { let pad = 0.5_f64.max(lo.abs() * 0.05); lo -= pad; hi += pad; }
            if lo < 0.0 && hi > 0.0 {
                (lo, hi, 0.0, 1.0 / lo.abs().max(hi.abs()), "diverging")
            } else if hi <= 0.0 {
                (lo, hi, -lo, 1.0 / (hi - lo), "sequential")
            } else {
                (lo, hi, -lo, 1.0 / (hi - lo), "sequential")
            }
        } else { (-1.0, 1.0, 0.0, 1.0, "diverging") };

        let mut out = format!(
            "{{\"schema_version\":1,\"kind\":\"{}\",\"width\":{},\"height\":{},\"labels\":[",
            if self.kind == "correlation" { "correlation_plot" } else { "general_matrix_plot" },
            fmt(width), fmt(height)
        );
        for (i, label) in self.labels.iter().enumerate() {
            if i > 0 { out.push(','); }
            let cy = top + i as f64 * cell + cell / 2.0;
            let cx = left + i as f64 * cell + cell / 2.0;
            out.push_str(&format!(
                "{{\"text\":\"{}\",\"row_x\":{},\"row_y\":{},\"column_x\":{},\"column_y\":{},\"row_visible\":true,\"column_visible\":true}}",
                esc(label), fmt(left - 12.0), fmt(cy), fmt(cx), fmt(top - 12.0)
            ));
        }
        out.push_str("],\"cells\":[");
        let mut first = true;
        for r in 0..n {
            for c in 0..n {
                if !first { out.push(','); }
                first = false;
                let value = self.values[r][c];
                let visible = self.display == "full" || (self.display == "upper" && r <= c) || (self.display == "lower" && r >= c);
                let region = if r == c { "diagonal" } else if r < c { "upper" } else { "lower" };
                let color = match value { None => &self.missing, Some(v) if v > 0.0 => &self.positive, Some(v) if v < 0.0 => &self.negative, _ => &self.neutral };
                let x = left + c as f64 * cell;
                let y = top + r as f64 * cell;
                let cx = x + cell / 2.0;
                let cy = y + cell / 2.0;
                out.push_str(&format!(
                    "{{\"row\":{r},\"column\":{c},\"value\":{},\"x\":{},\"y\":{},\"center_x\":{},\"center_y\":{},\"size\":{},\"color\":\"{}\",\"visible\":{},\"method\":\"{}\",\"region\":\"{}\"",
                    value.map(fmt).unwrap_or_else(|| "null".into()), fmt(x), fmt(y), fmt(cx), fmt(cy), fmt(cell), color, visible, self.method, region
                ));
                if self.kind == "general_numeric" {
                    let normalized = value.map(|v| (v + intercept) * zoom).map(fmt).unwrap_or_else(|| "null".into());
                    out.push_str(&format!(",\"normalized_value\":{normalized}"));
                }
                out.push('}');
            }
        }
        out.push_str(&format!(
            "],\"annotation\":\"\",\"title\":\"{}\",\"description\":\"Generated from publication-config v1 by corrplot-ng CLI\",\"glyph_method\":\"{}\",\"display_type\":\"{}\",\"show_diagonal\":true",
            esc(title), self.method, self.display
        ));
        out.push_str(&format!(
            ",\"style\":{{\"positive_color\":\"{}\",\"negative_color\":\"{}\",\"zero_color\":\"{}\",\"missing_fill\":\"{}\",\"grid_color\":\"#e0e0e0\"}}",
            self.positive, self.negative, self.neutral, self.missing
        ));
        out.push_str(&format!(
            ",\"typography\":{{\"label_font_family\":\"{}\",\"label_font_size\":{},\"label_font_weight\":\"normal\",\"column_label_orientation\":\"vertical\",\"label_collision\":\"preserve\"}}",
            esc(&self.family), fmt(self.base_size)
        ));
        out.push_str(&format!(
            ",\"legend\":[{{\"kind\":\"stop\",\"label\":\"-1\",\"color\":\"{}\"}},{{\"kind\":\"stop\",\"label\":\"0\",\"color\":\"{}\"}},{{\"kind\":\"stop\",\"label\":\"1\",\"color\":\"{}\"}},{{\"kind\":\"missing\",\"label\":\"Missing\",\"color\":\"{}\"}}]",
            self.negative, self.neutral, self.positive, self.missing
        ));
        out.push_str(&format!(
            ",\"variable_order\":{{\"strategy\":\"{}\",\"indices\":[",
            self.ordering
        ));
        for (i, index) in self.order_indices.iter().enumerate() { if i > 0 { out.push(','); } out.push_str(&index.to_string()); }
        out.push_str("],\"labels\":[");
        for (i, label) in self.original_labels.iter().enumerate() { if i > 0 { out.push(','); } out.push('"'); out.push_str(&esc(label)); out.push('"'); }
        out.push_str("]}");
        if self.kind == "general_numeric" {
            out.push_str(&format!(
                ",\"matrix_semantics\":{{\"mode\":\"general_numeric\",\"color_limits\":[{},{}],\"color_mapping\":\"{}\",\"normalization\":{{\"intercept\":{},\"zoom\":{}}}}}",
                fmt(lower), fmt(upper), mapping, fmt(intercept), fmt(zoom)
            ));
        }
        if self.method == "number" || self.show_values {
            out.push_str(",\"text_annotations\":[");
            let mut first = true;
            for r in 0..n { for c in 0..n {
                if !((self.display == "full") || (self.display == "upper" && r <= c) || (self.display == "lower" && r >= c)) { continue; }
                if let Some(value) = self.values[r][c] {
                    if !first { out.push(','); } first = false;
                    out.push_str(&format!(
                        "{{\"row\":{r},\"column\":{c},\"role\":\"value\",\"text\":\"{}\",\"color\":\"#ffffff\",\"font_size\":{},\"font_weight\":\"normal\",\"font_style\":\"normal\"}}",
                        fmt(value), fmt(self.base_size)
                    ));
                }
            }}
            out.push(']');
        }
        out.push('}');
        out
    }

    pub fn svg(&self, title: &str) -> String {
        // SVG is deliberately a renderer of the canonical renderer-neutral scene.
        // Keep matrix semantics and ordering in scene construction, not here.
        let scene = parse_json(&self.scene_json(title)).expect("scene builder must emit valid JSON");
        let root = match scene { Json::Object(ref o) => o, _ => unreachable!() };
        let get = scene_get;
        let num = scene_num;
        let strv = scene_str;
        let boolv = scene_bool;
        let esc = |v: &str| v.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;").replace('\'',"&apos;");
        let width = num(get(root,"width"));
        let height = num(get(root,"height"));
        let title_text = strv(get(root,"title"));
        let labels = match get(root,"labels") { Json::Array(a) => a, _ => unreachable!() };
        let cells = match get(root,"cells") { Json::Array(a) => a, _ => unreachable!() };
        let style = match get(root,"style") { Json::Object(o) => o, _ => unreachable!() };
        let typography = match get(root,"typography") { Json::Object(o) => o, _ => unreachable!() };
        let family = strv(get(typography,"label_font_family"));
        let font_size = num(get(typography,"label_font_size"));
        let mut out = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}{}\" height=\"{}{}\" viewBox=\"0 0 {} {}\">\n<title>{}</title>\n",
            self.width,self.unit,self.height,self.unit,width,height,esc(title_text)
        );
        out.push_str("<rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n");
        out.push_str(&format!(
            "<text x=\"120\" y=\"28\" font-family=\"{}\" font-size=\"{}\" font-weight=\"bold\">{}</text>\n",
            esc(family),font_size*1.5,esc(title_text)
        ));
        for label in labels {
            let o = match label { Json::Object(o) => o, _ => unreachable!() };
            let text = esc(strv(get(o,"text")));
            if boolv(get(o,"row_visible")) {
                out.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"end\" dominant-baseline=\"middle\" font-family=\"{}\" font-size=\"{}\">{}</text>\n",
                    num(get(o,"row_x")),num(get(o,"row_y")),esc(family),font_size,text
                ));
            }
            if boolv(get(o,"column_visible")) {
                out.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"{}\" font-size=\"{}\">{}</text>\n",
                    num(get(o,"column_x")),num(get(o,"column_y")),esc(family),font_size,text
                ));
            }
        }
        let annotations = root.get("text_annotations").and_then(|v| match v { Json::Array(a) => Some(a.as_slice()), _ => None }).unwrap_or(&[]);
        for cell in cells {
            let o = match cell { Json::Object(o) => o, _ => unreachable!() };
            if !boolv(get(o,"visible")) { continue; }
            let x=num(get(o,"x")); let y=num(get(o,"y")); let size=num(get(o,"size"));
            let cx=num(get(o,"center_x")); let cy=num(get(o,"center_y"));
            let color=strv(get(o,"color"));
            out.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\"/>\n",
                x,y,size,size,esc(color),esc(strv(get(style,"grid_color")))
            ));
            if matches!(get(o,"value"),Json::Null) { continue; }
            let value=num(get(o,"value"));
            let method=strv(get(o,"method"));
            if method != "number" {
                let normalized = o.get("normalized_value").map(num).unwrap_or_else(|| value.abs());
                let radius=normalized.abs().min(1.0)*27.0+2.0;
                out.push_str(&format!(
                    "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>\n",
                    cx,cy,radius,esc(color)
                ));
            }
            for annotation in annotations {
                let a = match annotation { Json::Object(a) => a, _ => continue };
                if num(get(a,"row")) as usize == num(get(o,"row")) as usize
                    && num(get(a,"column")) as usize == num(get(o,"column")) as usize {
                    out.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\">{}</text>\n",
                        cx,cy,esc(family),num(get(a,"font_size")),esc(strv(get(a,"color"))),esc(strv(get(a,"text")))
                    ));
                }
            }
        }
        out.push_str("</svg>\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_json_unicode_and_rejects_trailing_content() {
        assert!(parse_json(r#"{"x":"λ","y":[1,null,true]}"#).is_ok());
        assert!(parse_json("{} trailing").is_err());
    }
    #[test]
    fn publication_scene_matches_scene_v1_structure_for_both_matrix_kinds() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../prototypes/publication-config-v1/examples/basic.json");
        let source = std::fs::read_to_string(&fixture).unwrap();
        for (kind, expected_scene_kind) in [
            ("correlation", "correlation_plot"),
            ("general_numeric", "general_matrix_plot"),
        ] {
            let config = if kind == "general_numeric" {
                source.replace(
                    "\"kind\": \"correlation\", \"method\": \"pearson\"",
                    "\"kind\": \"general_numeric\"",
                )
            } else {
                source.clone()
            };
            let path = std::env::temp_dir().join(format!(
                "corrplot-ng-scene-contract-{}-{kind}.json",
                std::process::id()
            ));
            std::fs::write(&path, config).unwrap();
            let publication = Publication::load(path.to_str().unwrap()).unwrap();
            let scene = parse_json(&publication.scene_json("contract test")).unwrap();
            let root = object(&scene, "scene").unwrap();
            keys(
                root,
                &[
                    "schema_version", "kind", "width", "height", "labels", "cells",
                    "annotation", "title", "description", "style", "typography", "legend",
                    "glyph_method", "display_type", "show_diagonal", "variable_order",
                    "matrix_semantics", "text_annotations",
                ],
                "scene",
            ).unwrap();

            assert_eq!(number(required(root, "schema_version", "scene").unwrap(), "schema_version").unwrap(), 1.0);
            assert_eq!(string(required(root, "kind", "scene").unwrap(), "kind").unwrap(), expected_scene_kind);
            let labels = array(required(root, "labels", "scene").unwrap(), "labels").unwrap();
            let cells = array(required(root, "cells", "scene").unwrap(), "cells").unwrap();
            assert_eq!(labels.len(), publication.labels.len());
            assert_eq!(cells.len(), publication.labels.len() * publication.labels.len());

            for label in labels {
                let label = object(label, "label").unwrap();
                keys(label, &["text", "row_x", "row_y", "column_x", "column_y", "row_visible", "column_visible"], "label").unwrap();
                for key in ["text", "row_x", "row_y", "column_x", "column_y"] {
                    assert!(label.contains_key(key), "scene label missing {key}");
                }
            }
            for cell in cells {
                let cell = object(cell, "cell").unwrap();
                keys(cell, &["row", "column", "value", "x", "y", "center_x", "center_y", "size", "color", "visible", "method", "region", "normalized_value"], "cell").unwrap();
                for key in ["row", "column", "value", "x", "y", "center_x", "center_y", "size"] {
                    assert!(cell.contains_key(key), "scene cell missing {key}");
                }
                assert!(matches!(cell.get("value"), Some(Json::Number(_)) | Some(Json::Null)));
            }
            let style = object(required(root, "style", "scene").unwrap(), "style").unwrap();
            keys(style, &["positive_color", "negative_color", "zero_color", "missing_fill", "grid_color"], "style").unwrap();
            for key in ["positive_color", "negative_color", "zero_color", "missing_fill", "grid_color"] {
                assert!(style.contains_key(key), "scene style missing {key}");
            }
            let typography = object(required(root, "typography", "scene").unwrap(), "typography").unwrap();
            keys(typography, &["label_font_family", "label_font_size", "label_font_weight", "column_label_orientation", "label_collision"], "typography").unwrap();
            let legend = array(required(root, "legend", "scene").unwrap(), "legend").unwrap();
            for entry in legend {
                let entry = object(entry, "legend entry").unwrap();
                keys(entry, &["kind", "label", "color"], "legend entry").unwrap();
            }
            let order = object(required(root, "variable_order", "scene").unwrap(), "variable_order").unwrap();
            keys(order, &["strategy", "indices", "labels"], "variable_order").unwrap();
            if kind == "general_numeric" {
                let semantics = object(required(root, "matrix_semantics", "scene").unwrap(), "matrix_semantics").unwrap();
                keys(semantics, &["mode", "color_limits", "color_mapping", "normalization"], "matrix_semantics").unwrap();
                let normalization = object(required(semantics, "normalization", "matrix_semantics").unwrap(), "normalization").unwrap();
                keys(normalization, &["intercept", "zoom"], "normalization").unwrap();
                assert_eq!(string(required(semantics, "mode", "matrix_semantics").unwrap(), "mode").unwrap(), "general_numeric");
            } else {
                assert!(!root.contains_key("matrix_semantics"));
            }
            std::fs::remove_file(path).ok();
        }
    }

    #[test]
    fn general_negative_values_normalize_from_zero_to_one() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../prototypes/publication-config-v1/examples/basic.json");
        let source = std::fs::read_to_string(&fixture).unwrap();
        let config = source
            .replace(
                "\"kind\": \"correlation\", \"method\": \"pearson\"",
                "\"kind\": \"general_numeric\"",
            )
            .replace("[1.0, -0.42, 0.71]", "[-10.0, -5.0, -2.0]")
            .replace("[-0.42, 1.0, -0.18]", "[-5.0, -8.0, -3.0]")
            .replace("[0.71, -0.18, 1.0]", "[-2.0, -3.0, -6.0]");
        let path = std::env::temp_dir().join(format!(
            "corrplot-ng-negative-normalization-{}.json",
            std::process::id()
        ));
        std::fs::write(&path, config).unwrap();
        let publication = Publication::load(path.to_str().unwrap()).unwrap();
        let scene = parse_json(&publication.scene_json("normalization test")).unwrap();
        let root = object(&scene, "scene").unwrap();
        let cells = array(required(root, "cells", "scene").unwrap(), "cells").unwrap();

        let normalized_at = |row: usize, column: usize| {
            let cell = object(&cells[row * publication.labels.len() + column], "cell").unwrap();
            match cell.get("normalized_value") {
                Some(Json::Number(value)) => *value,
                other => panic!("expected normalized numeric value, got {other:?}"),
            }
        };
        assert!((normalized_at(0, 0) - 0.0).abs() < 1e-12);
        assert!((normalized_at(0, 2) - 1.0).abs() < 1e-12);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn aoe_order_is_deterministic_and_uses_original_index_mapping() {
        let values = vec![
            vec![Some(1.0), Some(-0.2), Some(0.0)],
            vec![Some(-0.2), Some(1.0), Some(0.9)],
            vec![Some(0.0), Some(0.9), Some(1.0)],
        ];
        let expected = vec![1, 2, 0];
        assert_eq!(aoe_order(&values).unwrap(), expected);
        assert_eq!(aoe_order(&values).unwrap(), expected);
    }

    #[test]
    fn aoe_rejects_missing_and_degenerate_correlations() {
        let missing = vec![vec![Some(1.0), None], vec![None, Some(1.0)]];
        assert!(aoe_order(&missing).unwrap_err().contains("requires complete correlations"));
        let degenerate = vec![
            vec![Some(1.0), Some(0.0), Some(0.0)],
            vec![Some(0.0), Some(1.0), Some(0.0)],
            vec![Some(0.0), Some(0.0), Some(1.0)],
        ];
        assert!(aoe_order(&degenerate).unwrap_err().contains("eigenvalues are tied"));
    }

    #[test]
    fn hclust_uses_average_linkage_and_original_index_ties() {
        let values = vec![
            vec![Some(1.0), Some(-0.2), Some(0.95)],
            vec![Some(-0.2), Some(1.0), Some(0.0)],
            vec![Some(0.95), Some(0.0), Some(1.0)],
        ];
        assert_eq!(hclust_order(&values).unwrap(), vec![0, 2, 1]);
    }

    #[test]
    fn hclust_omits_missing_pairs_but_rejects_isolated_variables() {
        let connected = vec![
            vec![Some(1.0), Some(0.4), None],
            vec![Some(0.4), Some(1.0), Some(0.5)],
            vec![None, Some(0.5), Some(1.0)],
        ];
        assert_eq!(hclust_order(&connected).unwrap(), vec![0, 1, 2]);
        let isolated = vec![
            vec![Some(1.0), None, None],
            vec![None, Some(1.0), Some(0.2)],
            vec![None, Some(0.2), Some(1.0)],
        ];
        assert!(hclust_order(&isolated).unwrap_err().contains("no observed off-diagonal correlation"));
    }

}
