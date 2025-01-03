use anyhow::{anyhow, Result};
use regex::Regex;
use sql_builder::{quote, SqlBuilder};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

static VALUE_REGEX: &str = r"^[a-zA-Z0-9_]+$";

#[derive(Debug, Clone, Copy, Display, PartialEq, EnumString, Hash)]
#[strum(serialize_all = "camelCase")]
pub enum Operator {
	Eq,
	Ne,
	Gt,
	Lt,
	Ge,
	Le,
	Contains,
	StartsWith,
	EndsWith,
	In,
}

impl Operator {
	pub fn to_sql(self) -> &'static str {
		match self {
			Operator::Eq => "=",
			Operator::Ne => "!=",
			Operator::Gt => ">",
			Operator::Lt => "<",
			Operator::Ge => ">=",
			Operator::Le => "<=",
			Operator::Contains => "LIKE",
			Operator::StartsWith => "LIKE",
			Operator::EndsWith => "LIKE",
			Operator::In => "IN",
		}
	}
}

#[derive(Debug, Clone, Display)]
pub enum FieldType {
	String,
	Integer,
	Float,
	Boolean,
	DateTime,
	Array(Box<FieldType>),
}

#[derive(Debug, Clone)]
pub struct FieldDefinition {
	pub name: String,
	pub field_type: FieldType,
}

#[derive(Debug, Clone, Hash)]
pub struct Condition {
	pub field: String,
	pub operator: Operator,
	pub value: String,
}

#[derive(Debug, Clone, Hash)]
pub struct Filter {
	pub conditions: Vec<Condition>,
}

impl Filter {
	pub fn new(value: String, field_definitions: &[FieldDefinition]) -> Result<Self> {
		let conditions = value
			.split(',')
			.map(|condition| {
				let parts: Vec<&str> = condition.split(':').collect();
				if parts.len() != 3 {
					return Err(anyhow!("Invalid condition format"));
				}

				let field = parts[0].to_string();
				let operator = Operator::from_str(parts[1])?;
				let value = parts[2].to_string();

				let field_def = field_definitions
					.iter()
					.find(|def| def.name == field)
					.ok_or_else(|| anyhow!("Unknown field: {}", field))?;

				validate_operator(&field_def.field_type, &operator)?;
				validate_value(&field_def.field_type, &value)?;

				Ok(Condition {
					field,
					operator,
					value,
				})
			})
			.collect::<Result<Vec<_>>>()?;

		Ok(Filter { conditions })
	}

	pub fn apply_to_query_builder(&self, builder: &mut SqlBuilder) {
		self.conditions.iter().for_each(|condition| {
			match condition.operator {
				Operator::Eq | Operator::Ne | Operator::Gt | Operator::Lt | Operator::Ge | Operator::Le => {
					let condition = format!(
						"{} {} {}",
						&condition.field,
						&condition.operator.to_sql(),
						quote(&condition.value)
					);

					builder.and_where(condition)
				}
				Operator::Contains => builder.and_where_like_any(&condition.field, quote(&condition.value)),
				Operator::StartsWith => {
					builder.and_where_like_left(&condition.field, quote(&condition.value))
				}
				Operator::EndsWith => {
					builder.and_where_like_right(&condition.field, quote(&condition.value))
				}
				Operator::In => builder.and_where(format!(
					"hasAny({}, {})",
					condition.field,
					condition
						.value
						.split("|")
						.map(quote)
						.collect::<Vec<String>>()
						.join(",")
				)),
			};
		});
	}
}

fn validate_operator(field_type: &FieldType, operator: &Operator) -> Result<()> {
	match field_type {
		FieldType::String => {
			if !matches!(
				operator,
				Operator::Eq
					| Operator::Ne
					| Operator::Contains
					| Operator::StartsWith
					| Operator::EndsWith
			) {
				return Err(anyhow!("Invalid operator for string field"));
			}
		}
		FieldType::Integer | FieldType::Float => {
			if !matches!(
				operator,
				Operator::Eq | Operator::Ne | Operator::Gt | Operator::Lt | Operator::Ge | Operator::Le
			) {
				return Err(anyhow!("Invalid operator for numeric field"));
			}
		}
		FieldType::Boolean => {
			if !matches!(operator, Operator::Eq | Operator::Ne) {
				return Err(anyhow!("Invalid operator for boolean field"));
			}
		}
		FieldType::DateTime => {
			if !matches!(
				operator,
				Operator::Eq | Operator::Ne | Operator::Gt | Operator::Lt | Operator::Ge | Operator::Le
			) {
				return Err(anyhow!("Invalid operator for datetime field"));
			}
		}
		FieldType::Array(_) => {
			if !matches!(operator, Operator::In) {
				return Err(anyhow!("Invalid operator for array field"));
			}
		}
	}
	Ok(())
}

fn validate_value(field_type: &FieldType, value: &str) -> Result<()> {
	match field_type {
		FieldType::String => {
			let re = Regex::new(VALUE_REGEX)?;

			if re.is_match(value) {
				Ok(())
			} else {
				Err(anyhow!("Invalid value"))
			}
		}
		FieldType::Integer => {
			value.parse::<i64>()?;
			Ok(())
		}
		FieldType::Float => {
			value.parse::<f64>()?;
			Ok(())
		}
		FieldType::Boolean => {
			value.parse::<bool>()?;
			Ok(())
		}
		FieldType::DateTime => {
			// TODO: implement datetime validation
			Ok(())
		}
		FieldType::Array(inner_type) => {
			for item in value.split('|') {
				validate_value(inner_type, item)?;
			}
			Ok(())
		}
	}
}
