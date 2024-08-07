CREATE TABLE Tasks(
    id SERIAL PRIMARY KEY,
    schedule_at TIMESTAMPTZ NOT NULL,                    
    picked_at_by_producers TIMESTAMPTZ[] NOT NULL DEFAULT '{}', 
    picked_at_by_workers TIMESTAMPTZ[] NOT NULL DEFAULT '{}',
    successful_at TIMESTAMPTZ DEFAULT NULL,            
    failed_ats TIMESTAMPTZ[] NOT NULL DEFAULT '{}',      
    failed_reasons TEXT[] NOT NULL DEFAULT '{}',          
    total_retry SMALLINT CHECK (total_retry >= 0 AND total_retry <= 3) NOT NULL,
    tracing_id VARCHAR(256) NOT NULL,                     
    current_retry SMALLINT NOT NULL DEFAULT 0,
    file_uploaded BOOLEAN NOT NULL DEFAULT FALSE,
    is_producible BOOLEAN NOT NULL DEFAULT TRUE
);


CREATE INDEX idx_schedule_at ON Tasks(schedule_at);
