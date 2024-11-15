CREATE INDEX idx_advertisement_cond ON advertisement(country, platform, gender) INCLUDE (age_range, end_at);
CREATE INDEX idx_advertisement_block ON advertisement USING BRIN(age_range, end_at);