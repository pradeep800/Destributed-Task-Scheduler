
CREATE TYPE status AS ENUM ('COMPLETED', 'PROCESSING', 'FAILED','ADDED');

CREATE TABLE Tasks (
    id SERIAL PRIMARY KEY, 
    schedule_at_in_second INTEGER NOT NULL,
    status status NOT NULL, 
    output TEXT NOT NULL,  
    retry SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL 
);

