CREATE TABLE advertisement
(
    id        SERIAL PRIMARY KEY,
    title     VARCHAR(255) NOT NULL,
    age_range INT4RANGE    NULL,
    country   INT          NULL,
    platform  INT          NULL,
    end_at    TIMESTAMP    NOT NULL
);
