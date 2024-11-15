CREATE TABLE advertisement
(
    id        SERIAL PRIMARY KEY,
    title     VARCHAR(255) NOT NULL,
    age_range INT4RANGE    NULL,
    country   int4         NULL,
    platform  int4         NULL,
    gender    int4         NULL,
    end_at    TIMESTAMP    NOT NULL
);
