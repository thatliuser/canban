# canban
Import your Canvas assignments into a Notion database.
This is currently quite barebones and doesn't handle errors robustly.

# Quickstart
- Clone this repo.
- Copy the provided config.sample.json to config.json.
- Go to your school's Canvas and generate an API token.
    - Go to Account > Approved Integrations > New Access Token and set up one.
- Go to your Notion and generate an internal integration.
    - Go to https://www.notion.so/profile/integrations and create a new integration (make sure the type is "internal").
- Go to your Notion database and connect it to your integration, so it has access.
    - Go to the Notion database page > 3 dots at the top right > Connections > Connect to > Your integration.
- Create the subjects in the Notion database and alias each Canvas class ID in the config.json file.
- Run this program with `cargo run`.
- If you did this correctly and the app didn't get rate limited, you should have all your assignments imported into your database.
- Run this again when you want to update anything.
