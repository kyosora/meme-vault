#[derive(Debug, Clone)]
pub struct SearchFilter {
  pub clause: String,
  pub params: Vec<String>,
}

#[derive(Debug, Default)]
struct Group {
  include: Vec<String>,
  exclude: Vec<String>,
}

pub fn build_filter(query: Option<&str>) -> Option<SearchFilter> {
  let raw_query = query.unwrap_or("").trim();
  if raw_query.is_empty() {
    return None;
  }

  let mut groups: Vec<Group> = Vec::new();

  for raw_group in raw_query.split('|') {
    let normalized = raw_group.replace('+', " ");
    let mut group = Group::default();

    for token in normalized.split_whitespace() {
      if let Some(tag) = token.strip_prefix('-') {
        let trimmed = tag.trim();
        if !trimmed.is_empty() {
          group.exclude.push(trimmed.to_string());
        }
      } else {
        group.include.push(token.to_string());
      }
    }

    if !group.include.is_empty() || !group.exclude.is_empty() {
      groups.push(group);
    }
  }

  if groups.is_empty() {
    return None;
  }

  let mut params = Vec::new();
  let mut clauses = Vec::new();

  for group in groups {
    let mut group_clauses: Vec<String> = Vec::new();

    for name in group.include {
      group_clauses.push(
        "EXISTS (SELECT 1 FROM image_tags it JOIN tags t ON t.id = it.tag_id WHERE it.image_id = i.id AND t.name = ?)".to_string(),
      );
      params.push(name);
    }

    for name in group.exclude {
      group_clauses.push(
        "NOT EXISTS (SELECT 1 FROM image_tags it JOIN tags t ON t.id = it.tag_id WHERE it.image_id = i.id AND t.name = ?)".to_string(),
      );
      params.push(name);
    }

    if !group_clauses.is_empty() {
      clauses.push(format!("({})", group_clauses.join(" AND ")));
    }
  }

  if clauses.is_empty() {
    return None;
  }

  Some(SearchFilter {
    clause: clauses.join(" OR "),
    params,
  })
}