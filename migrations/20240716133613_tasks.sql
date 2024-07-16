
CREATE TYPE status AS ENUM ('COMPLETED', 'PROCESSING', 'FAILED');

CREATE TABLE Tasks (
    id SERIAL PRIMARY KEY, 
    schedule_at_in_second INTEGER,
    status status, 
    yield TEXT,  
    retry SMALLINT,
    created_at TIMESTAMP  
);

