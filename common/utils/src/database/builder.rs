pub struct QueryBuilder {
	query: String,
	params: Vec<String>,
}

impl QueryBuilder {
	pub fn new() -> Self {
		QueryBuilder {
			query: String::new(),
			params: Vec::new(),
		}
	}

	pub fn select(mut self, columns: &[&str]) -> Self {
		self.query = format!("SELECT {} ", columns.join(", "));
		self
	}

	pub fn from(mut self, table: &str) -> Self {
		self.query += &format!("FROM {} ", table);
		self
	}

	pub fn where_clause(mut self, conditions: &[(String, String, String)]) -> Self {
		if !conditions.is_empty() {
			self.query += "WHERE ";
			let mut clauses = Vec::new();
			for (column, operator, value) in conditions {
				clauses.push(format!("{} {} ? ", column, operator));
				self.params.push(value.to_string());
			}
			self.query += &clauses.join(" AND ");
		}
		self
	}

	pub fn order_by(mut self, column: &str, direction: &str) -> Self {
		self.query += &format!("ORDER BY {} {} ", column, direction);
		self
	}

	pub fn limit(mut self, limit: usize) -> Self {
		self.query += &format!("LIMIT {} ", limit);
		self
	}

	pub fn build(self) -> (String, Vec<String>) {
		(self.query, self.params)
	}
}

impl Default for QueryBuilder {
	fn default() -> Self {
		Self::new()
	}
}
