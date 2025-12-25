lexer grammar CronLexer;

WS : [ \t\r\n]+ -> skip ;

CRON_COMMENT : '#' ~[\r\n]* -> skip ;

STAR        : '*';
SLASH       : '/';
COMMA       : ',';
DASH        : '-';

MONTH_NAME
    : 'JAN' | 'FEB' | 'MAR' | 'APR' | 'MAY' | 'JUN'
    | 'JUL' | 'AUG' | 'SEP' | 'OCT' | 'NOV' | 'DEC'
    ;

DOW_NAME
    : 'SUN' | 'MON' | 'TUE' | 'WED' | 'THU' | 'FRI' | 'SAT'
    ;

INT : [0-9]+ ;

COMMAND : ~[\r\n]* ;