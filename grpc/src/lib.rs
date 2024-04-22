mod services {
    use {
        entities::sea_orm::{DbErr, SqlErr},
        lool::s,
    };

    pub mod market;
    pub mod ticker;

    pub(crate) fn handle_sql_err(err: DbErr, action: &str, entity_name: &str) -> tonic::Status {
        let sqlerr = err.sql_err();

        match sqlerr {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                tonic::Status::already_exists(format!("{} already exists", entity_name))
            }
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                tonic::Status::failed_precondition(s!("Related entity does not exist"))
            }
            _ => tonic::Status::internal(format!("Error {} {}", action, entity_name)),
        }
    }
}

pub mod server;
