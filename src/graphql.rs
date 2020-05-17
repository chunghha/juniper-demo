use juniper::http::GraphQLRequest;
use juniper::RootNode;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_postgres::Client;

#[derive(juniper::GraphQLObject)]
struct Customer {
    id: String,
    name: String,
    age: i32,
    email: String,
    address: String,
}

pub struct QueryRoot;
pub struct MutationRoot;

pub struct Context {
    pub db_client: Client,
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

impl juniper::Context for Context {}

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    async fn customer(ctx: &Context, id: String) -> juniper::FieldResult<Customer> {
        let uuid = uuid::Uuid::parse_str(&id)?;
        let row = ctx
            .db_client
            .query_one(
                "SELECT name, age, email, address FROM customers WHERE id = $1",
                &[&uuid],
            )
            .await?;
        let customer = Customer {
            id,
            name: row.try_get(0)?,
            age: row.try_get(1)?,
            email: row.try_get(2)?,
            address: row.try_get(3)?,
        };
        info!("Customer {:#?} searched by {:#?}.", &customer.email, &uuid);
        Ok(customer)
    }

    async fn customers(ctx: &Context) -> juniper::FieldResult<Vec<Customer>> {
        let rows = ctx
            .db_client
            .query("SELECT id, name, age, email, address FROM customers", &[])
            .await?;
        let mut customers = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.try_get(0)?;
            let customer = Customer {
                id: id.to_string(),
                name: row.try_get(1)?,
                age: row.try_get(2)?,
                email: row.try_get(3)?,
                address: row.try_get(4)?,
            };
            customers.push(customer);
        }
        info!("{:#?} customer(s) returned.", customers.len());
        Ok(customers)
    }
}

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    async fn register_customer(
        ctx: &Context,
        name: String,
        age: i32,
        email: String,
        address: String,
    ) -> juniper::FieldResult<Customer> {
        let id = uuid::Uuid::new_v4();
        let email = email.to_lowercase();
        ctx.db_client
            .execute(
                "INSERT INTO customers (id, name, age, email, address) VALUES ($1, $2, $3, $4, $5)",
                &[&id, &name, &age, &email, &address],
            )
            .await?;
        info!("{:#?} created for {:#?}.", &id, &email);
        Ok(Customer {
            id: id.to_string(),
            name,
            age,
            email,
            address,
        })
    }

    async fn update_customer_email(
        ctx: &Context,
        id: String,
        email: String,
    ) -> juniper::FieldResult<String> {
        let uuid = uuid::Uuid::parse_str(&id)?;
        let email = email.to_lowercase();
        let n = ctx
            .db_client
            .execute(
                "UPDATE customers SET email = $1 WHERE id = $2",
                &[&email, &uuid],
            )
            .await?;
        if n == 0 {
            return Err("User does not exist".into());
        }
        info!("{:#?} updated for {:#?}.", &uuid, &email);
        Ok(email)
    }

    async fn delete_customer(ctx: &Context, id: String) -> juniper::FieldResult<bool> {
        let uuid = uuid::Uuid::parse_str(&id)?;
        let n = ctx
            .db_client
            .execute("DELETE FROM customers WHERE id = $1", &[&uuid])
            .await?;
        if n == 0 {
            return Err("User does not exist".into());
        }
        info!("{:#?} deleted.", &id);
        Ok(true)
    }
}

pub async fn graphql_resolve(
    schema: Arc<Schema>,
    ctx: Arc<Context>,
    req: GraphQLRequest,
) -> Result<impl warp::Reply, Infallible> {
    let res = req.execute_async(&schema, &ctx).await;
    let json = serde_json::to_string(&res).expect("Invalid JSON response");
    Ok(json)
}
